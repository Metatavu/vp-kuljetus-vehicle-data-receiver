mod utils;
mod teltonika_handler;
mod telematics_cache;
mod vehicle_management_service;
mod teltonika_connection;

use std::error::Error;
use log::info;
use tokio::net::TcpListener;

use crate::teltonika_connection::TeltonikaConnection;

/// Reads string environment variable
///
/// Panics if the environment variable is not set.
///
/// # Arguments
/// * `key` - Environment variable key
///
/// # Returns
/// * `String` - Environment variable value
fn read_string_env_variable(key: &str) -> String {
    match std::env::var(key) {
        Ok(value) => value,
        Err(_) => panic!("{} environment variable not set", key)
    }
}

/// Reads boolean environment variable
///
/// Panics if the environment variable is not set.
///
/// # Arguments
/// * `key` - Environment variable key
///
/// # Returns
/// * `bool` - Environment variable value
fn read_bool_env_variable(key: &str) -> bool {
    match std::env::var(key) {
        Ok(value) => value.parse().unwrap(),
        Err(_) => panic!("{} environment variable not set", key)
    }
}

/// VP-Kuljetus Vehicle Data Receiver
///
/// This application handles incoming TCP connections from Teltonika Telematics devices,
/// processes the data and sends it to the VP-Kuljetus Vehicle Management Service API.
///
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    env_logger::init();
    let file_path = read_string_env_variable("BASE_FILE_PATH");
    let write_to_file = read_bool_env_variable("WRITE_TO_FILE");

    // This is retrieved from the environment on-demand but we want to restrict starting the software if the environment variable is not set
    read_string_env_variable("VEHICLE_MANAGEMENT_SERVICE_CLIENT_API_KEY");

    // Generated client gets the base URL from the environment variable itself but we want to restrict starting the software if the environment variable is not set
    read_string_env_variable("VEHICLE_MANAGEMENT_SERVICE_CLIENT_BASE_URL");

    let address = "127.0.0.1:8080";

    let listener = TcpListener::bind(&address).await?;

    info!("Listening on: {}", address);

    loop {
        let (socket, _) = listener.accept().await?;
        let base_file_path = match write_to_file {
          true => file_path.clone(),
          false => "".to_string()
        };

        tokio::spawn(async move {
            let mut teltonika_connection = TeltonikaConnection::new(socket);
            if let Err(_) = teltonika_connection.handle_connection(base_file_path).await {
                return;
            };
        });
    }
}

#[cfg(test)]
mod tests {
    use httpmock::{Method::GET, MockServer};
    use log::error;
    use nom_teltonika::{parser, AVLEventIO, Priority};
    use tempfile::tempdir;
    use vehicle_management_service_client::model::PublicTruck;
    use crate::{
        utils::{
            avl_frame_builder::*,
            avl_packet::*,
            avl_record_builder::avl_record_builder::*,
            imei::*,
            str_to_bytes
        },
        vehicle_management_service::VehicleManagementService
    };
    use self::{telematics_cache::{cacheable_truck_speed::CacheableTruckSpeed, Cacheable}, teltonika_handler::TeltonikaRecordsHandler};

    use super::*;

    #[test]
    fn test_valid_imei() {
        let generated_imei_1 = get_random_imei_of_length(10);
        let generated_imei_2 = get_random_imei_of_length(15);
        let imei_packet_1 = build_valid_imei_packet(&generated_imei_1);
        let imei_packet_2 = build_valid_imei_packet(&generated_imei_2);
        let read_imei_result_1 = read_imei(&imei_packet_1);
        let read_imei_result_2 = read_imei(&imei_packet_2);

        let is_first_imei_valid = read_imei_result_1.0;
        let is_second_imei_valid = read_imei_result_2.0;
        let parsed_first_imei = read_imei_result_1.1.unwrap();
        let parsed_second_imei = read_imei_result_2.1.unwrap();

        assert_eq!(is_first_imei_valid, true);
        assert_eq!(is_second_imei_valid, true);
        assert_eq!(&parsed_first_imei, &generated_imei_1.clone());
        assert_eq!(&parsed_second_imei, &generated_imei_2.clone());
    }

    #[test]
    fn test_invalid_imei() {
        let generated_imei = get_random_imei_of_length(15);
        let imei_packet = build_invalid_imei_packet(&generated_imei);
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
            .with_io_events(vec![
                AVLEventIO {
                    id: 10,
                    value: nom_teltonika::AVLEventIOValue::U8(10),
                }
            ])
            .build();
        let packet = AVLFrameBuilder::new()
            .add_record(record)
            .build()
            .to_bytes();

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
        let record_handler = get_teltonika_records_handler(None);
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
        let packet_with_record_without_vin = AVLFrameBuilder::new()
            .add_record(record_without_vin)
            .build();

        let missing_vin = record_handler.get_truck_vin_from_records(&packet_with_record_without_vin.records);

        assert_eq!(missing_vin, None);
    }

    #[test]
    fn test_partly_missing_truck_vin() {
        let record_handler = get_teltonika_records_handler(None);
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
        let packet_with_record_without_vin = AVLFrameBuilder::new()
            .add_record(record_without_vin)
            .build();

        let missing_vin = record_handler.get_truck_vin_from_records(&packet_with_record_without_vin.records);

        assert_eq!(missing_vin, None);
    }

    #[test]
    fn test_get_truck_vin() {
        let record_handler = get_teltonika_records_handler(None);
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
        let packet_with_record_with_vin = AVLFrameBuilder::new()
            .add_record(record_with_vin)
            .build();

        let vin = record_handler.get_truck_vin_from_records(&packet_with_record_with_vin.records);

        assert_eq!("W1T96302X10704959", vin.unwrap());
    }

    #[test]
    fn test_get_truck_vin_with_multiple_vin_records() {
        let record_handler = get_teltonika_records_handler(None);
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

        let vin = record_handler.get_truck_vin_from_records(&packet_with_multiple_records_with_vin.records);

        assert_eq!("W1T96302X10704959", vin.unwrap());
    }

    #[tokio::test]
    async fn test_get_truck_id_with_valid_vin() {
        start_vehicle_management_mock();
        let vin = Some(String::from("W1T96302X10704959"));
        let truck_id = VehicleManagementService::get_truck_id_by_vin(&vin).await;

        assert!(truck_id.is_some());
        assert_eq!("3FFAF18C-69E4-4F8A-9179-9AEC5BC96E1C", truck_id.unwrap());
    }

    #[tokio::test]
    async fn test_get_truck_id_with_invalid_vin() {
        start_vehicle_management_mock();
        let vin = Some(String::from("invalid-vin"));
        let truck_id = VehicleManagementService::get_truck_id_by_vin(&vin).await;

        assert!(truck_id.is_none());
    }

    #[test]
    fn test_cache_speed_event() {
        let record_handler = get_teltonika_records_handler(None);
        let record = AVLRecordBuilder::new()
            .with_priority(Priority::High)
            .with_io_events(vec![
                AVLEventIO {
                    id: 191,
                    value: nom_teltonika::AVLEventIOValue::U16(10),
                },
            ])
            .build();
        let packet = AVLFrameBuilder::new()
            .add_record(record)
            .build();

        record_handler.handle_records(packet.records);

        let base_cache_path = record_handler.get_cache_path();
        let speeds_cache = CacheableTruckSpeed::read_from_file(base_cache_path.to_str().unwrap());
        let first_cached_speed = speeds_cache.first();

        assert_eq!(1, speeds_cache.len());
        assert_eq!(10.0, first_cached_speed.unwrap().speed);
    }

    /// Reads IMEI from the buffer
    ///
    /// # Arguments
    /// * `buffer` - Buffer for reading data from socket
    ///
    /// # Returns
    /// * `(bool, Option<String>)` - Whether the IMEI was successfully parsed and the IMEI itself as an `Option<String>`
    fn read_imei(buffer: &Vec<u8>) -> (bool, Option<String>) {
        let result = nom_teltonika::parser::imei(&buffer);
        match result {
            Ok((_, imei)) => {
                info!("New client connected with IMEI: [{:?}]", imei);
                return (true, Some(imei));
            },
            Err(_) => {
                error!("Failed to parse IMEI from buffer");
                return (false, None);
            }
        }
    }

    /// Gets a TeltonikaRecordsHandler for testing
    ///
    /// Uses a temporary directory for the cache
    fn get_teltonika_records_handler(truck_id: Option<String>) -> TeltonikaRecordsHandler {
        let test_cache_dir = tempdir().unwrap();
        let test_cache_path = test_cache_dir.path();

        return TeltonikaRecordsHandler::new( test_cache_path, truck_id);
    }

    /// Starts a mock server for the Vehicle Management Service
    fn start_vehicle_management_mock() {
        let mock_server = MockServer::start();
        let mut server_address = String::from("http://");
        server_address.push_str(mock_server.address().to_string().as_str());

        std::env::set_var("VEHICLE_MANAGEMENT_SERVICE_CLIENT_BASE_URL", &server_address);
        std::env::set_var("VEHICLE_MANAGEMENT_SERVICE_CLIENT_API_KEY", "API_KEY");

        let _public_trucks_mock = mock_server.mock(|when, then| {
            when.method(GET)
                .path("/vehicle-management/v1/publicTrucks")
                .header("X-API-KEY", "API_KEY");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body_obj(&[PublicTruck{
                    id: Some(String::from("3FFAF18C-69E4-4F8A-9179-9AEC5BC96E1C")),
                    name: Some(String::from("1")),
                    plate_number: String::from("ABC-123"),
                    vin: String::from("W1T96302X10704959"),
                }]);
        });
    }
}