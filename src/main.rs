use std::error::Error;
use std::fs::OpenOptions;
use std::net::SocketAddr;
use std::io::Write;
use log::{debug, error, info};
use nom_teltonika::parser;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// VP-Kuljetus Vehicle Data Receiver
///
/// This application handles incoming TCP connections from Teltonika Telematics devices,
/// processes the data and sends it to the VP-Kuljetus Vehicle Management Service API.
///
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    env_logger::init();
    let address = "0.0.0.0:8080";

    let listener = TcpListener::bind(&address).await?;

    info!("Listening on: {}", address);

    loop {
        let (mut socket, socket_address) = listener.accept().await?;

        info!("New client: [{}]", socket_address);


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
                write_all_to_socket(&mut socket, &b"\x00".to_vec()).await.unwrap();
                socket.shutdown().await.expect("Failed to shutdown socket");
                return;
            } else {
                write_all_to_socket(&mut socket, &b"\x01".to_vec()).await.unwrap();
            }

            if let Result::Err(err) = handle_valid_connection(socket, &mut buffer, socket_address, imei).await {
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
async fn write_all_to_socket(socket: &mut TcpStream, buffer: &Vec<u8>) -> Result<(), Box<dyn Error>> {
    socket.write_all(&buffer)
        .await
        .expect(&format!("Failed to write {:#?} to socket", buffer));
    Ok(())
}

/// Handles individual TCP connection from Teltonika Telematics device
///
/// For local development and debugging purposes, this currently stores the data in a file named after the IMEI of the device.
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
    imei: Option<String>
) -> Result<(), Box<dyn Error>> {
    let mut file = OpenOptions::new().read(true).create(true).append(true).open(format!("{}.txt", imei.unwrap())).expect("Failed to open file");
    loop {
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
        writeln!(file, "{:#?}", frame).unwrap();
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
            info!("Parsed IMEI: [{:?}]", imei);
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
    use rand::{distributions::Alphanumeric, Rng};
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

    fn build_valid_imei_packet(imei: &str) -> Vec<u8> {
        let length = imei.len() as i16;
        let mut imei_byte_array = length.to_be_bytes().to_vec();

        imei_byte_array.append(&mut imei.as_bytes().to_vec());

        return imei_byte_array;
    }

    fn build_invalid_imei_packet(imei: &str) -> Vec<u8> {
        return imei.as_bytes().to_vec();
    }

    fn get_random_imei_of_length(length: i16) -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(length as usize)
            .map(char::from)
            .collect()
    }
}