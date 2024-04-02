mod test_utils;
mod teltonika_handler;

use std::error::Error;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::Write;
use chrono::{Datelike, Utc};
use log::{debug, error, info};
use nom_teltonika::parser;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use vehicle_management_service::{VehicleManagementServiceAuth, VehicleManagementServiceClient};

use crate::teltonika_handler::TeltonikaRecordsHandler;

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
    let file_path = read_string_env_variable("LOG_FILE_PATH");
    let write_to_file = read_bool_env_variable("WRITE_TO_FILE");

    // This is retrieved from the environment on-demand but we want to restrict starting the software if the environment variable is not set
    read_string_env_variable("VEHICLE_MANAGEMENT_SERVICE_API_KEY");

    // Generated client gets the base URL from the environment variable itself but we want to restrict starting the software if the environment variable is not set
    read_string_env_variable("VEHICLE_MANAGEMENT_SERVICE_BASE_URL");

    let address = "0.0.0.0:8080";

    let listener = TcpListener::bind(&address).await?;

    info!("Listening on: {}", address);

    loop {
        let (mut socket, _) = listener.accept().await?;
        let log_file_path = match write_to_file {
          true => file_path.clone(),
          false => "".to_string()
        };

        tokio::spawn(async move {
            let mut buffer = vec![0; 4096];
            let n = socket
                .read(&mut buffer)
                .await
                .expect("Failed to read data from socket");

            if n == 0 {
                return;
            }

            let (valid_imei, imei) = read_imei(&buffer);

            if  !valid_imei {
                write_all_to_socket(&mut socket, &[0x00]).await.unwrap();
                socket.shutdown().await.expect("Failed to shutdown socket");
                return;
            } else {
                write_all_to_socket(&mut socket, &[0x01]).await.unwrap();
            }

            if let Result::Err(err) = handle_valid_connection(
                socket,
                &mut buffer,
                imei.unwrap(),
                log_file_path,
            ).await {
                error!("Error processing connection: {}", err);
            };
        });
    }
}

/// Writes buffer to socket
///
/// # Arguments
/// * `socket` - TCP socket
/// * `buffer` - Buffer to write to socket
async fn write_all_to_socket(socket: &mut TcpStream, buffer: &[u8]) -> Result<(), Box<dyn Error>> {
    socket.write_all(&buffer)
        .await
        .expect(&format!("Failed to write {:#?} to socket", buffer));
    debug!("Wrote {:02X?} to socket", buffer);
    Ok(())
}

/// Gets file handle for log file
///
/// # Arguments
/// * `imei` - IMEI of the Teltonika Telematics device
/// * `log_file_path` - Path to log file
///
/// # Returns
/// * `Option<File>` - File handle
fn get_log_file_handle(imei: &str, log_file_path: &str) -> Option<File> {
    if cfg!(not(test)) && log_file_path != "" {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let parent_path = std::path::Path::new(log_file_path).join(imei);
        create_dir_all(&parent_path).expect(&format!("Failed to create log file directory `{:#?}`", &parent_path));
        return Some(
            OpenOptions::new()
                .read(true)
                .create(true)
                .append(true)
                .open(
                    parent_path.join(format!("{}.bin", today))
                )
                .expect("Failed to open file")
        );
    }

    return None;
}

/// Write data to log file
///
/// # Arguments
/// * `file_handle` - File handle
/// * `data` - Data to write to file
fn write_data_to_log_file(file_handle: &mut Option<File>, data: &[u8]) {
    if cfg!(test) {
        return;
    }
    if let Some(file) = file_handle {
        file.write_all(data).expect("Failed to write data to file");
    }
}

/// Gets Truck ID from the VP-Kuljetus Vehicle Management Service
/// If the Truck ID is not found, None is returned
///
/// # Arguments
/// * `vin` - VIN of the truck
///
/// # Returns
/// * `Option<String>` - Truck ID
async fn get_truck_id(vin: &Option<String>) -> Option<String> {
    if vin.is_none() {
        return None;
    }

    let vehicle_management_service_api_key = read_string_env_variable("VEHICLE_MANAGEMENT_SERVICE_API_KEY");

    let vehicle_management_service_client = VehicleManagementServiceClient::with_auth(
        VehicleManagementServiceAuth::ApiKeyAuth { x_api_key: vehicle_management_service_api_key }
    );
    let trucks = vehicle_management_service_client.list_public_trucks().await.expect("Failed to list public trucks");

    for truck in trucks {
        if truck.vin == vin.clone().unwrap() {
            return truck.id;
        }
    }

    return None;
}

/// Handles individual TCP connection from Teltonika Telematics device
///
/// # Arguments
/// * `socket` - TCP socket
/// * `buffer` - Buffer for reading data from socket
/// * `imei` - IMEI of the Teltonika Telematics device
/// * `log_file_path` - Path to log file
async fn handle_valid_connection(
    mut socket: TcpStream,
    buffer: &mut Vec<u8>,
    imei: String,
    log_file_path: String,
) -> Result<(), Box<dyn Error>> {
    let teltonika_records_handler = TeltonikaRecordsHandler::new();
    let start_of_connection = Utc::now();

    let mut file_handle = get_log_file_handle(&imei, &log_file_path);
    let mut truck_vin: Option<String> = None;
    let mut truck_id: Option<String> = None;

    loop {
        let start_of_loop = Utc::now();
        if start_of_loop.day() != start_of_connection.day() {
            file_handle = get_log_file_handle(&imei, &log_file_path);
        }
        let n = socket
            .read(buffer)
            .await
            .expect("Failed to read data from socket");

        if n == 0 {
            break;
        }

        let first_byte = buffer[0];

        if first_byte == 0xFF {
            debug!("Received ping from IMEI {}", imei);
            continue;
        }
        let (_, frame) = parser::tcp_frame(&buffer).expect("Failed to parse TCP frame");
        let amount_of_records = frame.records.len();

        // If the truck VIN is not set, try to get it from the records
        if let None = &truck_vin {
            truck_vin = teltonika_records_handler.get_truck_vin_from_records(frame.records);
        }

        // If the truck ID is not set, try to get it from the VP-Kuljetus Vehicle Management Service
        if let None = truck_id {
            truck_id = get_truck_id(&truck_vin).await;
        }

        if let Some(vin) = &truck_vin {
            debug!("Received {:02X} records from VIN {} with IMEI {}", amount_of_records, vin, imei);
        } else {
            debug!("Received {:02X} records from unknown VIN with IMEI {}", amount_of_records, imei);
        }

        write_data_to_log_file(&mut file_handle, &buffer);

        socket.write_i32(amount_of_records as i32).await?;

        if let Some(vin) = &truck_vin {
            debug!("Sent {:02X} records to VIN {} with IMEI {}", amount_of_records as i32, vin, imei);
        } else {
            debug!("Sent {:02X} records to unknown VIN with IMEI {}", amount_of_records as i32, imei);
        }
    }
    info!("Client with IMEI {} disconnected", imei);

    Ok(())
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
            error!("Failed to parse IMEI from buffer {:#?}", buffer);
            return (false, None);
        }
    }
}

#[cfg(test)]
mod tests {
    use httpmock::{Method::GET, MockServer};
    use nom_teltonika::{AVLEventIO, Priority};
    use vehicle_management_service::model::PublicTruck;
    use crate::test_utils::{
        avl_frame_builder::*, avl_packet::*, avl_record_builder::avl_record_builder::*, imei::*, utilities::str_to_bytes
    };
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
        let record_handler = TeltonikaRecordsHandler::new();
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

        let missing_vin = record_handler.get_truck_vin_from_records(packet_with_record_without_vin.records);

        assert_eq!(missing_vin, None);
    }

    #[test]
    fn test_get_truck_vin() {
        let record_handler = TeltonikaRecordsHandler::new();
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

        let vin = record_handler.get_truck_vin_from_records(packet_with_record_with_vin.records);

        assert_eq!("W1T96302X10704959", vin.unwrap());
    }

    #[test]
    fn test_get_truck_vin_with_multiple_vin_records() {
        let record_handler = TeltonikaRecordsHandler::new();
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

        let vin = record_handler.get_truck_vin_from_records(packet_with_multiple_records_with_vin.records);

        assert_eq!("W1T96302X10704959", vin.unwrap());
    }

    #[tokio::test]
    async fn get_truck_id_with_valid_vin() {
        start_vehicle_management_mock();
        let vin = Some(String::from("W1T96302X10704959"));
        let truck_id = get_truck_id(&vin).await;

        assert!(truck_id.is_some());
        assert_eq!("3FFAF18C-69E4-4F8A-9179-9AEC5BC96E1C", truck_id.unwrap());
    }

    #[tokio::test]
    async fn get_truck_id_with_invalid_vin() {
        start_vehicle_management_mock();
        let vin = Some(String::from("invalid-vin"));
        let truck_id = get_truck_id(&vin).await;

        assert!(truck_id.is_none());
    }

    fn start_vehicle_management_mock() {
        let mock_server = MockServer::start();
        let mut server_address = String::from("http://");
        server_address.push_str(mock_server.address().to_string().as_str());

        std::env::set_var("VEHICLE_MANAGEMENT_SERVICE_BASE_URL", &server_address);
        std::env::set_var("VEHICLE_MANAGEMENT_SERVICE_API_KEY", "API_KEY");

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