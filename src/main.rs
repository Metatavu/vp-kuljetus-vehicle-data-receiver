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
    use crate::{
        telematics_cache::Cacheable,
        utils::{
            avl_frame_builder::*,
            avl_packet::*,
            avl_record_builder::avl_record_builder::*,
            get_vehicle_management_api_config,
            imei::{build_valid_imei_packet, get_random_imei_of_length, *},
            str_to_bytes,
            test_utils::{
                driver_card_id_to_two_part_events, driver_card_part_to_dec,
                get_teltonika_records_handler, read_imei, split_at_half,
                start_vehicle_management_mock, string_to_hex_string,
            },
        },
    };
    use nom_teltonika::{parser, AVLEventIO, Priority};
    use std::str::FromStr;
    use vehicle_management_service::{
        apis::public_trucks_api::ListPublicTrucksParams,
        models::{
            TruckDriveState, TruckDriveStateEnum, TruckDriverCard, TruckLocation, TruckSpeed,
        },
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
        let valid_driver_card_id = "1069619335000001".to_string();
        let valid_driver_card_id_2 = "1A696193350YZ001".to_string();
        start_vehicle_management_mock();
        let mut record_handler = get_teltonika_records_handler(None);
        let driver_card_events = driver_card_id_to_two_part_events(valid_driver_card_id.clone());
        let record = AVLRecordBuilder::new()
            .with_io_events(driver_card_events.to_vec())
            .add_io_event(AVLEventIO {
                id: 187,
                value: nom_teltonika::AVLEventIOValue::U8(1),
            })
            .with_trigger_event_id(187)
            .build();
        let packet = AVLFrameBuilder::new()
            .with_records([record].to_vec())
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
            assert_eq!(valid_driver_card_id, cached_driver_card_event.id);
        }
        record_handler.set_truck_id(Some("F8C5BC38-0213-487D-A37A-553AC3A9D77F".to_string()));
        record_handler.purge_cache().await;
        {
            let base_cache_path = record_handler.get_base_cache_path();
            let driver_cards_cache =
                TruckDriverCard::read_from_file(base_cache_path.to_str().unwrap());

            assert_eq!(0, driver_cards_cache.len());
        }
        // Test that a driver card id containing more than just numbers is also handled correctly
        record_handler.set_truck_id(None);
        let driver_card_events = driver_card_id_to_two_part_events(valid_driver_card_id_2.clone());
        let record = AVLRecordBuilder::new()
            .with_io_events(driver_card_events.to_vec())
            .add_io_event(AVLEventIO {
                id: 187,
                value: nom_teltonika::AVLEventIOValue::U8(1),
            })
            .with_trigger_event_id(187)
            .build();
        let packet = AVLFrameBuilder::new()
            .with_records([record].to_vec())
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
            assert_eq!(valid_driver_card_id_2, cached_driver_card_event.id);
        }
        record_handler.set_truck_id(Some("F8C5BC38-0213-487D-A37A-553AC3A9D77F".to_string()));
        record_handler.purge_cache().await;
        // Test that a record without 187 (driver 1 card presence) as a trigger event is not handled
        record_handler.set_truck_id(None);
        let driver_card_events = driver_card_id_to_two_part_events(valid_driver_card_id.clone());
        let record = AVLRecordBuilder::new()
            .with_io_events(driver_card_events.to_vec())
            .build();
        let packet = AVLFrameBuilder::new()
            .with_records([record].to_vec())
            .build();

        record_handler.handle_records(packet.records).await;

        {
            let base_cache_path = record_handler.get_base_cache_path();
            let driver_cards_cache =
                TruckDriverCard::read_from_file(base_cache_path.to_str().unwrap());
            assert_eq!(0, driver_cards_cache.len());
        }
    }

    #[tokio::test]
    async fn test_driver_one_card_drive_state_handling() {
        let valid_driver_card_id = "1069619335000001".to_string();
        start_vehicle_management_mock();
        let mut record_handler = get_teltonika_records_handler(None);
        let driver_card_events = driver_card_id_to_two_part_events(valid_driver_card_id.clone());
        let record_1 = AVLRecordBuilder::new()
            .with_io_events(driver_card_events.to_vec())
            .add_io_event(AVLEventIO {
                id: 184,
                value: nom_teltonika::AVLEventIOValue::U8(3),
            })
            .build();
        let packet = AVLFrameBuilder::new()
            .with_records([record_1].to_vec())
            .build();

        record_handler.handle_records(packet.records).await;

        {
            let base_cache_path = record_handler.get_base_cache_path();
            let driver_cards_cache =
                TruckDriveState::read_from_file(base_cache_path.to_str().unwrap());
            let cached_driver_card_event = driver_cards_cache.get(0);
            assert_eq!(1, driver_cards_cache.len());
            assert!(cached_driver_card_event.is_some());
            let cached_driver_card_event = cached_driver_card_event.unwrap();
            assert_eq!(
                valid_driver_card_id,
                cached_driver_card_event.driver_card_id.clone().unwrap()
            );
            assert_eq!(TruckDriveStateEnum::Drive, cached_driver_card_event.state);
        }
        record_handler.set_truck_id(Some("F8C5BC38-0213-487D-A37A-553AC3A9D77F".to_string()));
        record_handler.purge_cache().await;
        {
            let base_cache_path = record_handler.get_base_cache_path();
            let driver_cards_cache =
                TruckDriveState::read_from_file(base_cache_path.to_str().unwrap());

            assert_eq!(0, driver_cards_cache.len());
        }
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
        let driver_card_id_msb_dec = driver_card_part_to_dec(&driver_card_id_msb);
        let driver_card_id_lsb_dec = driver_card_part_to_dec(&driver_card_id_lsb);
        // Step 1 in the documentation
        assert_eq!(driver_card_id_msb_dec, 3544392526090811699);
        assert_eq!(driver_card_id_lsb_dec, 3689908453225017393);
    }
}
