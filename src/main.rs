mod failed_events;
mod teltonika;
mod utils;

use crate::utils::trackable_cache_item::TrackableCacheItem;
use crate::{teltonika::connection::TeltonikaConnection, utils::read_env_variable};
use futures::future::join_all;
use lazy_static::lazy_static;
use log::{debug, info, warn};
use rand::{thread_rng, Rng};
use sqlx::{migrate::Migrator, mysql::MySqlPoolOptions, MySql, Pool};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::{io::ErrorKind, time::Duration};
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use vp_kuljetus_vehicle_data_receiver::failed_events::FailedEventsHandler;
use vp_kuljetus_vehicle_data_receiver::listener::Listener;
use vp_kuljetus_vehicle_data_receiver::teltonika::records::TeltonikaRecordsHandler;
use vp_kuljetus_vehicle_data_receiver::utils::api::get_trackable;
use vp_kuljetus_vehicle_data_receiver::utils::read_env_variable_with_default_value;

const VEHICLE_MANAGEMENT_SERVICE_API_KEY_ENV_KEY: &str = "VEHICLE_MANAGEMENT_SERVICE_API_KEY";
const API_BASE_URL_ENV_KEY: &str = "API_BASE_URL";

lazy_static! {
    static ref LISTENERS: [Listener; 2] = [Listener::TeltonikaFMC234, Listener::TeltonikaFMC650];
}

/// Starts a listener
///
/// # Arguments
/// * `listener` - Listener
async fn start_listener(listener: Listener, trackables_cache: Arc<RwLock<Vec<TrackableCacheItem>>>) {
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

        let cache = trackables_cache.clone();
        tokio::spawn(async move {
            if let Err(error) = TeltonikaConnection::handle_connection(socket, &listener, cache).await {
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

    let mut futures: Vec<Pin<Box<dyn Future<Output = ()> + Send>>> = Vec::new();

    let trackables_cache = Arc::new(RwLock::new(Vec::new()));
    for listener in LISTENERS.iter() {
        futures.push(Box::pin(start_listener(*listener, trackables_cache.clone())));
    }

    join_all(futures).await;
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