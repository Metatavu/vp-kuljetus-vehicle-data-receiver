use std::error::Error;
use std::fs::OpenOptions;
use std::net::SocketAddr;
use std::io::Write;
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
    let address = "0.0.0.0:8080";

    let listener = TcpListener::bind(&address).await?;

    println!("Listening on: {}", address);

    loop {
        let (mut socket, socket_address) = listener.accept().await?;

        println!("New client: [{}]", socket_address);


        tokio::spawn(async move {
            let mut buffer = vec![0; 4096];
            let n = socket
                .read(&mut buffer)
                .await
                .expect("Failed to read data from socket");

            if n == 0 {
                return;
            }

            // TODO: Validate IMEI from backend
            let (valid_imei, imei) = read_imei(&buffer);
            if  !valid_imei {
                socket.write_all(b"\x00").await.expect("Failed to write data to socket");
                socket.shutdown().await.expect("Failed to shutdown socket");
                return;
            } else {
                socket.write_all(b"\x01").await.expect("Failed to write data to socket");
            }
            if let Result::Err(err) = handle_valid_connection(socket, &mut buffer, socket_address, imei).await {
                println!("Error processing connection: {}", err);
            };
        });
    }
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
            println!("Received ping from client {}", socket_address);
            continue;
        }
        let (_, frame) = parser::tcp_frame(&buffer).expect("Failed to parse TCP frame");
        let amount_of_records = frame.records.len();
        println!("Received {} records from client {}", amount_of_records, socket_address);
        writeln!(file, "{:#?}", frame).unwrap();
        socket.write_i32(amount_of_records as i32).await?;
        println!("Sent {:x} records to client {}", amount_of_records as i32, socket_address)
    }
    println!("Client {} disconnected", socket_address);

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
            println!("Connected IMEI: [{:?}]", imei);
            return (true, Some(imei));
        },
        Err(_) => {
            println!("Failed to parse IMEI from buffer {:#?}", buffer);
            return (false, None);
        }
    }
}