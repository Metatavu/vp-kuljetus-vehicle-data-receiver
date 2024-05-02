mod telematics_cache;
mod teltonika_connection;
mod teltonika_handler;
mod utils;

use log::info;
use std::error::Error;
use tokio::net::TcpListener;

use crate::{
    teltonika_connection::TeltonikaConnection,
    utils::{read_bool_env_variable, read_string_env_variable},
};

/// VP-Kuljetus Vehicle Data Receiver
///
/// This application handles incoming TCP connections from Teltonika Telematics devices,
/// processes the data and sends it to the VP-Kuljetus Vehicle Management Service API.
///
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let file_path = read_string_env_variable("BASE_FILE_PATH");
    let write_to_file = read_bool_env_variable("WRITE_TO_FILE");

    // This is retrieved from the environment on-demand but we want to restrict starting the software if the environment variable is not set
    read_string_env_variable("VEHICLE_MANAGEMENT_SERVICE_API_KEY");

    // Generated client gets the base URL from the environment variable itself but we want to restrict starting the software if the environment variable is not set
    read_string_env_variable("API_BASE_URL");

    let address = "0.0.0.0:8080";

    let listener = TcpListener::bind(&address).await?;

    info!("Listening on: {}", address);

    loop {
        let (socket, _) = listener.accept().await?;
        let base_file_path = match write_to_file {
            true => file_path.clone(),
            false => "".to_string(),
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
    use self::teltonika_handler::teltonika_records_handler::TeltonikaRecordsHandler;
    use super::*;
    use crate::{
        telematics_cache::Cacheable,
        utils::{
            avl_frame_builder::*,
            avl_packet::*,
            avl_record_builder::avl_record_builder::*,
            get_vehicle_management_api_config,
            imei::{build_valid_imei_packet, get_random_imei_of_length, *},
            str_to_bytes,
        },
    };
    use httpmock::{
        Method::{GET, POST},
        MockServer, Regex,
    };
    use log::error;
    use nom_teltonika::{parser, AVLEventIO, Priority};
    use std::str::FromStr;
    use tempfile::tempdir;
    use uuid::Uuid;
    use vehicle_management_service::{
        apis::public_trucks_api::ListPublicTrucksParams,
        models::{PublicTruck, TruckDriverCard, TruckLocation, TruckSpeed},
    };

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

        let missing_vin =
            record_handler.get_truck_vin_from_records(&packet_with_record_without_vin.records);

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

        let missing_vin =
            record_handler.get_truck_vin_from_records(&packet_with_record_without_vin.records);

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
        let packet_with_record_with_vin =
            AVLFrameBuilder::new().add_record(record_with_vin).build();

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

        let vin = record_handler
            .get_truck_vin_from_records(&packet_with_multiple_records_with_vin.records);

        assert_eq!("W1T96302X10704959", vin.unwrap());
    }

    #[tokio::test]
    async fn test_get_truck_id_with_valid_vin() {
        start_vehicle_management_mock();
        let vin = Some(String::from("W1T96302X10704959"));
        let truck = vehicle_management_service::apis::public_trucks_api::list_public_trucks(
            &get_vehicle_management_api_config(),
            ListPublicTrucksParams {
                vin: vin.clone(),
                first: None,
                max: None,
            },
        )
        .await
        .unwrap()
        .into_iter()
        .find(|truck| truck.vin == vin.clone().unwrap());

        assert!(truck.is_some());
        assert_eq!(
            uuid::Uuid::from_str("3FFAF18C-69E4-4F8A-9179-9AEC5BC96E1C").unwrap(),
            truck.unwrap().id.unwrap()
        );
    }

    #[tokio::test]
    async fn test_cache_speed_event() {
        let record_handler = get_teltonika_records_handler(None);
        let record = AVLRecordBuilder::new()
            .with_priority(Priority::High)
            .with_io_events(vec![AVLEventIO {
                id: 191,
                value: nom_teltonika::AVLEventIOValue::U16(10),
            }])
            .build();
        let packet = AVLFrameBuilder::new().add_record(record).build();

        record_handler.handle_records(packet.records).await;

        let base_cache_path = record_handler.get_base_cache_path();
        let speeds_cache = TruckSpeed::read_from_file(base_cache_path.to_str().unwrap());
        let first_cached_speed = speeds_cache.first();

        assert_eq!(1, speeds_cache.len());
        assert_eq!(10.0, first_cached_speed.unwrap().speed);
    }

    #[tokio::test]
    async fn test_send_cached_event() {
        start_vehicle_management_mock();
        let mut record_handler = get_teltonika_records_handler(None);
        let record = AVLRecordBuilder::new()
            .with_priority(Priority::High)
            .with_io_events(vec![AVLEventIO {
                id: 191,
                value: nom_teltonika::AVLEventIOValue::U16(10),
            }])
            .build();
        let packet = AVLFrameBuilder::new().add_record(record).build();

        record_handler.handle_records(packet.records).await;

        {
            let base_cache_path = record_handler.get_base_cache_path();
            let speeds_cache = TruckSpeed::read_from_file(base_cache_path.to_str().unwrap());
            let first_cached_speed = speeds_cache.first();

            assert_eq!(1, speeds_cache.len());
            assert_eq!(10.0, first_cached_speed.unwrap().speed);
        }
        record_handler.set_truck_id(Some("F8C5BC38-0213-487D-A37A-553AC3A9D77F".to_string()));
        record_handler.purge_cache().await;
        {
            let base_cache_path = record_handler.get_base_cache_path();
            let speeds_cache = TruckSpeed::read_from_file(base_cache_path.to_str().unwrap());
            assert_eq!(0, speeds_cache.len());
        }
    }

    #[tokio::test]
    async fn test_record_location_handling() {
        start_vehicle_management_mock();
        let mut record_handler = get_teltonika_records_handler(None);
        let record_1 = AVLRecordBuilder::new()
            .with_longitude(61.68779453479687)
            .with_latitude(27.27297030282335)
            .with_angle(810)
            .build();
        let record_2 = AVLRecordBuilder::new()
            .with_longitude(27.27297030282335)
            .with_latitude(61.68779453479687)
            .with_angle(180)
            .build();
        let packet = AVLFrameBuilder::new()
            .with_records([record_1, record_2].to_vec())
            .build();

        record_handler.handle_records(packet.records).await;

        {
            let base_cache_path = record_handler.get_base_cache_path();
            let locations_cache = TruckLocation::read_from_file(base_cache_path.to_str().unwrap());

            let location_1 = locations_cache
                .iter()
                .find(|location| location.heading == 810.0)
                .unwrap();
            let location_2 = locations_cache
                .iter()
                .find(|location| location.heading == 180.0)
                .unwrap();

            assert_eq!(2, locations_cache.len());
            assert_eq!(61.68779453479687, location_1.longitude);
            assert_eq!(27.27297030282335, location_1.latitude);
            assert_eq!(810.0, location_1.heading);
            assert_eq!(27.27297030282335, location_2.longitude);
            assert_eq!(61.68779453479687, location_2.latitude);
            assert_eq!(180.0, location_2.heading);
        }
        record_handler.set_truck_id(Some("F8C5BC38-0213-487D-A37A-553AC3A9D77F".to_string()));
        record_handler.purge_cache().await;
        {
            let base_cache_path = record_handler.get_base_cache_path();
            let locations_cache = TruckSpeed::read_from_file(base_cache_path.to_str().unwrap());
            assert_eq!(0, locations_cache.len());
        }
    }

    #[tokio::test]
    async fn test_driver_one_card_id_handling() {
        start_vehicle_management_mock();
        let mut record_handler = get_teltonika_records_handler(None);
        let driver_card_events = driver_card_id_to_two_part_events("DVF1232950483967".to_string());
        let record_1 = AVLRecordBuilder::new()
            .with_io_events(driver_card_events.to_vec())
            .build();
        let packet = AVLFrameBuilder::new()
            .with_records([record_1].to_vec())
            .build();

        record_handler.handle_records(packet.records).await;

        {
            let base_cache_path = record_handler.get_base_cache_path();
            let driver_cards_cache =
                TruckDriverCard::read_from_file(base_cache_path.to_str().unwrap());
            let cached_driver_card_event = driver_cards_cache.get(0);
            assert_eq!(1, driver_cards_cache.len());
            assert!(cached_driver_card_event.is_some());
            let cached_driver_card_event = cached_driver_card_event.unwrap();
            assert_eq!("DVF1232950483967", cached_driver_card_event.id);
        }
        record_handler.set_truck_id(Some("F8C5BC38-0213-487D-A37A-553AC3A9D77F".to_string()));
        record_handler.purge_cache().await;
        {
            let base_cache_path = record_handler.get_base_cache_path();
            let driver_cards_cache =
                TruckDriverCard::read_from_file(base_cache_path.to_str().unwrap());

            assert_eq!(0, driver_cards_cache.len());
        }
    }

    fn driver_card_id_to_two_part_events(driver_card_id: String) -> [AVLEventIO; 2] {
        let driver_card_id_bytes = driver_card_id.as_bytes();
        let driver_card_id_lsb = u64::from_be_bytes([
            driver_card_id_bytes[0],
            driver_card_id_bytes[1],
            driver_card_id_bytes[2],
            driver_card_id_bytes[3],
            driver_card_id_bytes[4],
            driver_card_id_bytes[5],
            driver_card_id_bytes[6],
            driver_card_id_bytes[7],
        ]);
        let driver_card_id_msb = u64::from_be_bytes([
            driver_card_id_bytes[8],
            driver_card_id_bytes[9],
            driver_card_id_bytes[10],
            driver_card_id_bytes[11],
            driver_card_id_bytes[12],
            driver_card_id_bytes[13],
            driver_card_id_bytes[14],
            driver_card_id_bytes[15],
        ]);
        let driver_card_id_msb_event = AVLEventIO {
            id: 195,
            value: nom_teltonika::AVLEventIOValue::U64(driver_card_id_msb),
        };
        let driver_card_id_lsb_event = AVLEventIO {
            id: 196,
            value: nom_teltonika::AVLEventIOValue::U64(driver_card_id_lsb),
        };
        return [driver_card_id_msb_event, driver_card_id_lsb_event];
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
            }
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

        return TeltonikaRecordsHandler::new(test_cache_path, truck_id);
    }

    /// Starts a mock server for the Vehicle Management Service
    fn start_vehicle_management_mock() {
        let mock_server = MockServer::start();
        let mut server_address = String::from("http://");
        server_address.push_str(mock_server.address().to_string().as_str());

        std::env::set_var("API_BASE_URL", &server_address);
        std::env::set_var("VEHICLE_MANAGEMENT_SERVICE_API_KEY", "API_KEY");

        let _public_trucks_mock = mock_server.mock(|when, then| {
            when.method(GET)
                .path("/vehicle-management/v1/publicTrucks")
                .header("X-API-KEY", "API_KEY");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body_obj(&[PublicTruck {
                    id: Some(Uuid::from_str("3FFAF18C-69E4-4F8A-9179-9AEC5BC96E1C").unwrap()),
                    name: Some(String::from("1")),
                    plate_number: String::from("ABC-123"),
                    vin: String::from("W1T96302X10704959"),
                }]);
        });

        let _create_truck_speed_mock = mock_server.mock(|when, then| {
            when.method(POST)
                .path_matches(Regex::new(r"/vehicle-management/v1/trucks/.{36}/speeds").unwrap())
                .header("X-API-KEY", "API_KEY");
            then.status(201);
        });
        let _create_truck_driver_card_mock = mock_server.mock(|when, then| {
            when.method(POST)
                .path_matches(
                    Regex::new(r"/vehicle-management/v1/trucks/.{36}/driverCards").unwrap(),
                )
                .header("X-API-KEY", "API_KEY");
            then.status(201)
                .header("Content-Type", "application/json")
                .json_body_obj(&TruckDriverCard { id: String::new() });
        });
    }
}
