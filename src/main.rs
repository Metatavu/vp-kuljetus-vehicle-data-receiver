mod failed_events;
mod teltonika;
mod utils;
mod worker;

use crate::{teltonika::connection::TeltonikaConnection, utils::read_env_variable};
use futures::future::join_all;
use lazy_static::lazy_static;
use log::{debug, info, warn};
use rand::{thread_rng, Rng};
use sqlx::{migrate::Migrator, mysql::MySqlPoolOptions, MySql, Pool};
use std::future::Future;
use std::pin::Pin;
use std::{io::ErrorKind, thread::sleep, time::Duration};
use tokio::net::TcpListener;
use vp_kuljetus_vehicle_data_receiver::failed_events::FailedEventsHandler;
use vp_kuljetus_vehicle_data_receiver::listener::Listener;
use vp_kuljetus_vehicle_data_receiver::teltonika::records::TeltonikaRecordsHandler;
use vp_kuljetus_vehicle_data_receiver::utils::api::get_trackable;
use vp_kuljetus_vehicle_data_receiver::utils::read_env_variable_with_default_value;

const VEHICLE_MANAGEMENT_SERVICE_API_KEY_ENV_KEY: &str = "VEHICLE_MANAGEMENT_SERVICE_API_KEY";
const API_BASE_URL_ENV_KEY: &str = "API_BASE_URL";
const DATABASE_HOST: &str = "DATABASE_HOST";
const DATABASE_PORT: &str = "DATABASE_PORT";
const DATABASE_USERNAME: &str = "DATABASE_USERNAME";
const DATABASE_PASSWORD: &str = "DATABASE_PASSWORD";
const DATABASE_NAME: &str = "DATABASE_NAME";

const FAILED_EVENTS_BATCH_SIZE_ENV_KEY: &str = "FAILED_EVENTS_BATCH_SIZE";
const DEFAULT_FAILED_EVENTS_BATCH_SIZE: u64 = 100;

lazy_static! {
    static ref LISTENERS: [Listener; 2] = [Listener::TeltonikaFMC234, Listener::TeltonikaFMC650];
}

/// Initializes the database connection pool
///
/// # Arguments
/// * `host` - The database host
/// * `port` - The database port
/// * `database_name` - The database name
/// * `username` - The database username
/// * `password` - The database password
///
/// # Returns
/// A future that resolves to the database connection pool
async fn init_db(
    host: String,
    port: u16,
    database_name: String,
    username: String,
    password: String,
) -> Result<Pool<MySql>, sqlx::Error> {
    info!("Initializing database connection pool: {}", host);

    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(10))
        .connect_with(
            sqlx::mysql::MySqlConnectOptions::new()
                .host(host.as_str())
                .port(port)
                .username(username.as_str())
                .password(password.as_str())
                .ssl_mode(sqlx::mysql::MySqlSslMode::Disabled)
                .database(database_name.as_str()),
        )
        .await?;

    info!("Running database migrations...");
    let migrations_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
    let migrator = Migrator::new(migrations_dir).await?;
    migrator.run(&pool).await?;

    info!("Database migrations completed successfully.");
    Ok(pool)
}

/// Starts a periodic task for reprocessing failed events
///
/// # Arguments
/// * `database_pool` - Database connection pool
///
/// # Returns
/// A future that resolves when the worker is stopped
async fn start_failed_events_worker(database_pool: Pool<MySql>) {
    tokio::spawn(async move {
        let batch_size =
            read_env_variable_with_default_value(FAILED_EVENTS_BATCH_SIZE_ENV_KEY, DEFAULT_FAILED_EVENTS_BATCH_SIZE);

        loop {
            sleep(Duration::from_secs(1));

            debug!("Checking for failed events to reprocess...");

            let failed_events_handler = FailedEventsHandler::new(database_pool.clone());
            let failed_imei = failed_events_handler.next_failed_imei().await.unwrap();
            if let Some(imei) = failed_imei {
                if let Some(trackable) = get_trackable(&imei).await {
                    debug!("Found trackable for IMEI {}", imei);

                    let failed_events = failed_events_handler
                        .list_failed_events(&imei, batch_size)
                        .await
                        .unwrap();
                    let identifier: u32 = thread_rng().r#gen();
                    let log_target = imei.clone() + "-" + identifier.to_string().as_str();
                    let records_handler =
                        TeltonikaRecordsHandler::new(log_target.clone(), Some(trackable.clone()), imei.clone());

                    debug!("Processing {} failed events for IMEI {}", failed_events.len(), imei);

                    for failed_event in failed_events {
                        let failed_event_id = failed_event.id.unwrap();

                        let result = records_handler.handle_failed_event(failed_event).await;
                        if result.is_ok() {
                            match failed_events_handler.delete_failed_event(failed_event_id).await {
                                Ok(_) => debug!("Successfully processed failed event for IMEI {}", imei),
                                Err(e) => warn!("Failed to delete failed event for IMEI {}: {:?}", imei, e),
                            }
                        } else {
                            warn!("Failed to process failed event for IMEI {}: {:?}", imei, result.err());

                            match failed_events_handler
                                .update_attempted_at(
                                    failed_event_id,
                                    chrono::Utc::now().naive_utc().and_utc().timestamp(),
                                )
                                .await
                            {
                                Ok(_) => {
                                    debug!("Successfully updated attempted_at for failed event {}", failed_event_id)
                                }
                                Err(e) => warn!(
                                    "Failed to update attempted_at for failed event {}: {:?}",
                                    failed_event_id, e
                                ),
                            };
                        }
                    }
                } else {
                    info!("No trackable found for IMEI {}", imei);
                }
            } else {
                info!("No failed events found");
            }
        }
    });
}

/// Starts a listener
///
/// # Arguments
/// * `listener` - Listener
async fn start_listener(listener: Listener, database_pool: Pool<MySql>) {
    let address = format!("0.0.0.0:{}", listener.port());
    let tcp_listener = match TcpListener::bind(&address).await {
        Ok(l) => l,
        Err(e) => {
            panic!("Failed to bind to address: {}", e);
        }
    };

    info!("Listening on: {}", address);

    loop {
        let socket = match tcp_listener.accept().await {
            Ok((sock, _)) => sock,
            Err(e) => {
                panic!("Failed to accept connection: {}", e);
            }
        };

        let pool_clone = database_pool.clone();
        tokio::spawn(async move {
            if let Err(error) = TeltonikaConnection::handle_connection(socket, &listener, pool_clone).await {
                match error.kind() {
                    ErrorKind::ConnectionAborted | ErrorKind::InvalidData => {
                        warn!("Connection aborted: {}", error);
                    }
                    _ => {
                        return;
                    }
                }
            };
        });
    }
}
/// VP-Kuljetus Vehicle Data Receiver
///
/// This application handles incoming TCP connections from Teltonika Telematics devices,
/// processes the data and sends it to the VP-Kuljetus Vehicle Management Service API.
///
#[tokio::main]
async fn main() {
    env_logger::init();

    info!("Starting Vehicle Data Receiver...");

    // This is retrieved from the environment on-demand but we want to restrict starting the software if the environment variable is not set
    read_env_variable::<String>(VEHICLE_MANAGEMENT_SERVICE_API_KEY_ENV_KEY);

    // // Generated client gets the base URL from the environment variable itself but we want to restrict starting the software if the environment variable is not set
    read_env_variable::<String>(API_BASE_URL_ENV_KEY);

    // Initialize database connection pool and run migrations
    let database_pool = init_db(
        read_env_variable::<String>(DATABASE_HOST),
        read_env_variable::<u16>(DATABASE_PORT),
        read_env_variable::<String>(DATABASE_NAME),
        read_env_variable::<String>(DATABASE_USERNAME),
        read_env_variable::<String>(DATABASE_PASSWORD),
    )
    .await
    .expect("Failed to run migrations");

    let mut futures: Vec<Pin<Box<dyn Future<Output = ()> + Send>>> = Vec::new();
    let cron = start_failed_events_worker(database_pool.clone());
    futures.push(Box::pin(cron));

    for listener in LISTENERS.iter() {
        futures.push(Box::pin(start_listener(*listener, database_pool.clone())));
    }

    join_all(futures).await;

    database_pool.close().await;
}

#[cfg(test)]
mod tests {
    use crate::{
        teltonika::records::teltonika_vin_handler::get_truck_vin_from_records,
        utils::{
            avl_frame_builder::*,
            avl_packet::*,
            avl_record_builder::avl_record_builder::*,
            imei::{build_valid_imei_packet, get_random_imei, *},
            str_to_bytes,
            test_utils::{read_imei, split_at_half, string_to_hex_string, string_to_hex_to_dec},
        },
    };
    use nom_teltonika::{parser, AVLEventIO, Priority};

    #[test]
    fn test_valid_imei() {
        let random_imei_1 = get_random_imei();
        let random_imei_2 = get_random_imei();
        let imei_packet_1 = build_valid_imei_packet(&random_imei_1);
        let imei_packet_2 = build_valid_imei_packet(&random_imei_2);
        let (is_imei1_valid_by_nom, imei1) = read_imei(&imei_packet_1);
        let (is_imei2_valid_by_nom, imei2) = read_imei(&imei_packet_2);

        assert_eq!(is_imei1_valid_by_nom, imei::valid(imei1.unwrap()));
        assert_eq!(is_imei2_valid_by_nom, imei::valid(imei2.unwrap()));
    }

    #[test]
    fn test_invalid_imei() {
        let random_imei = get_random_imei();
        let imei_packet = build_invalid_imei_packet(&random_imei);
        let read_imei_result = read_imei(&imei_packet);

        let is_imei_valid = read_imei_result.0;
        let parsed_imei = read_imei_result.1;

        assert_eq!(is_imei_valid, false);
        assert_eq!(parsed_imei, None);
    }

    #[test]
    fn test_valid_packet() {
        let record = AVLRecordBuilder::new()
            .with_priority(Priority::Panic)
            .with_io_events(vec![AVLEventIO {
                id: 10,
                value: nom_teltonika::AVLEventIOValue::U8(10),
            }])
            .build();
        let packet = AVLFrameBuilder::new().add_record(record).build().to_bytes();

        let example_packet_str = "000000000000003608010000016B40D8EA30010000000000000000000000000000000105021503010101425E0F01F10000601A014E0000000000000000010000C7CF";
        let example_packet = str_to_bytes(example_packet_str);

        let parsed_built_packet = parser::tcp_frame(&packet);
        let parsed_example_packet = parser::tcp_frame(&example_packet);

        assert!(parsed_built_packet.is_ok());
        assert!(parsed_example_packet.is_ok());
    }

    #[test]
    #[should_panic]
    fn test_invalid_packet() {
        // This packet is missing the preamble
        let example_packet_str = "3608010000016B40D8EA30010000000000000000000000000000000105021503010101425E0F01F10000601A014E0000000000000000010000C7CF";
        let example_packet = str_to_bytes(example_packet_str);
        let parsed_example_packet = parser::tcp_frame(&example_packet);
        // This should panic because the packet is missing the preamble
        parsed_example_packet.unwrap();
    }

    #[test]
    fn test_missing_truck_vin() {
        let record_without_vin = AVLRecordBuilder::new()
            .with_priority(Priority::High)
            .with_io_events(vec![
                AVLEventIO {
                    id: 191,
                    value: nom_teltonika::AVLEventIOValue::U16(10),
                },
                AVLEventIO {
                    id: 1,
                    value: nom_teltonika::AVLEventIOValue::U16(20),
                },
                AVLEventIO {
                    id: 200,
                    value: nom_teltonika::AVLEventIOValue::U16(20),
                },
            ])
            .build();
        let packet_with_record_without_vin = AVLFrameBuilder::new().add_record(record_without_vin).build();

        let missing_vin = get_truck_vin_from_records(&packet_with_record_without_vin.records);

        assert_eq!(missing_vin, None);
    }

    #[test]
    fn test_partly_missing_truck_vin() {
        let record_without_vin = AVLRecordBuilder::new()
            .with_priority(Priority::High)
            .with_io_events(vec![
                AVLEventIO {
                    id: 234,
                    value: nom_teltonika::AVLEventIOValue::U64(6354913562786543925),
                },
                AVLEventIO {
                    id: 233,
                    value: nom_teltonika::AVLEventIOValue::U64(6282895559857745970),
                },
                AVLEventIO {
                    id: 200,
                    value: nom_teltonika::AVLEventIOValue::U16(20),
                },
            ])
            .build();
        let packet_with_record_without_vin = AVLFrameBuilder::new().add_record(record_without_vin).build();

        let missing_vin = get_truck_vin_from_records(&packet_with_record_without_vin.records);

        assert_eq!(missing_vin, None);
    }

    #[test]
    fn test_get_truck_vin() {
        let record_with_vin = AVLRecordBuilder::new()
            .with_priority(Priority::High)
            .with_io_events(vec![
                AVLEventIO {
                    id: 234,
                    value: nom_teltonika::AVLEventIOValue::U64(6354913562786543925),
                },
                AVLEventIO {
                    id: 233,
                    value: nom_teltonika::AVLEventIOValue::U64(6282895559857745970),
                },
                AVLEventIO {
                    id: 235,
                    value: nom_teltonika::AVLEventIOValue::U8(57),
                },
            ])
            .build();
        let packet_with_record_with_vin = AVLFrameBuilder::new().add_record(record_with_vin).build();

        let vin = get_truck_vin_from_records(&packet_with_record_with_vin.records);

        assert_eq!("W1T96302X10704959", vin.unwrap());
    }

    #[test]
    fn test_get_truck_vin_with_multiple_vin_records() {
        let record_with_vin_1 = AVLRecordBuilder::new()
            .with_priority(Priority::High)
            .with_io_events(vec![
                AVLEventIO {
                    id: 234,
                    value: nom_teltonika::AVLEventIOValue::U64(6354913562786543925),
                },
                AVLEventIO {
                    id: 233,
                    value: nom_teltonika::AVLEventIOValue::U64(6282895559857745970),
                },
                AVLEventIO {
                    id: 235,
                    value: nom_teltonika::AVLEventIOValue::U8(57),
                },
            ])
            .build();
        let record_with_vin_2 = AVLRecordBuilder::new()
            .with_priority(Priority::High)
            .with_io_events(vec![
                AVLEventIO {
                    id: 234,
                    value: nom_teltonika::AVLEventIOValue::U64(6354913562786543925),
                },
                AVLEventIO {
                    id: 233,
                    value: nom_teltonika::AVLEventIOValue::U64(6282895559857745970),
                },
                AVLEventIO {
                    id: 235,
                    value: nom_teltonika::AVLEventIOValue::U8(57),
                },
            ])
            .build();
        let packet_with_multiple_records_with_vin = AVLFrameBuilder::new()
            .with_records([record_with_vin_1, record_with_vin_2].to_vec())
            .build();

        let vin = get_truck_vin_from_records(&packet_with_multiple_records_with_vin.records);

        assert_eq!("W1T96302X10704959", vin.unwrap());
    }

    /// Tests the conversion of a driver card ID to two part events as described in [Teltonika documentation](https://wiki.teltonika-gps.com/view/DriverID)
    ///
    /// Field tests have proven that the conversion formula provided by Teltonika is incorrect and the ASCII strings should NOT be reversed.
    /// Therefore the conversion is done without reversing the ASCII strings.
    ///
    /// Note for later: Can the difference in conversion be due to different generations of digital tachographs? Customer will provide us with the model of the digital tachograph this test was done with.
    #[test]
    fn test_driver_card_conversion() {
        // Step 5 in the documentation
        let valid_driver_card_id = String::from("1069619335000001");
        let (driver_card_id_msb, driver_card_id_lsb) = split_at_half(valid_driver_card_id.clone());
        // Step 4 in the documentation
        assert_eq!(driver_card_id_msb, "10696193");
        assert_eq!(driver_card_id_lsb, "35000001");
        let driver_card_id_msb_hex = string_to_hex_string(&driver_card_id_msb);
        let driver_card_id_lsb_hex = string_to_hex_string(&driver_card_id_lsb);
        // Step 2 in the documentation
        assert_eq!(driver_card_id_msb_hex, "3130363936313933");
        assert_eq!(driver_card_id_lsb_hex, "3335303030303031");
        let driver_card_id_msb_dec = string_to_hex_to_dec(&driver_card_id_msb);
        let driver_card_id_lsb_dec = string_to_hex_to_dec(&driver_card_id_lsb);
        // Step 1 in the documentation
        assert_eq!(driver_card_id_msb_dec, 3544392526090811699);
        assert_eq!(driver_card_id_lsb_dec, 3689908453225017393);
    }
}
