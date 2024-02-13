mod test_utils;

use std::error::Error;
use std::fs::{create_dir_all, File, OpenOptions};
use std::net::SocketAddr;
use std::io::Write;
use chrono::{Datelike, Utc};
use log::{debug, error, info};
use nom_teltonika::parser;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};


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

    let address = "0.0.0.0:8080";

    let listener = TcpListener::bind(&address).await?;

    info!("Listening on: {}", address);

    loop {
        let (mut socket, socket_address) = listener.accept().await?;
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
                socket_address,
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

/// Handles individual TCP connection from Teltonika Telematics device
///
/// # Arguments
/// * `socket` - TCP socket
/// * `buffer` - Buffer for reading data from socket
/// * `socket_address` - Socket address of the client
/// * `imei` - IMEI of the Teltonika Telematics device
async fn handle_valid_connection(
    mut socket: TcpStream,
    buffer: &mut Vec<u8>,
    socket_address: SocketAddr,
    imei: String,
    log_file_path: String,
) -> Result<(), Box<dyn Error>> {
    let start_of_connection = Utc::now();
    let mut file_handle = get_log_file_handle(&imei, &log_file_path);
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
            debug!("Received ping from client {}", socket_address);
            continue;
        }
        let (_, frame) = parser::tcp_frame(&buffer).expect("Failed to parse TCP frame");
        let amount_of_records = frame.records.len();
        debug!("Received {} records from client {}", amount_of_records, socket_address);

        write_data_to_log_file(&mut file_handle, &buffer);

        socket.write_i32(amount_of_records as i32).await?;
        debug!("Sent {:x} records to client {}", amount_of_records as i32, socket_address)
    }
    info!("Client {} disconnected", socket_address);

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
    use nom_teltonika::{AVLEventIO, Priority};
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
}