use base64::Engine;
use chrono::{Datelike, Utc};
use log::{debug, error, info};
use nom_teltonika::TeltonikaStream;
use std::{
    fs::{create_dir_all, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::mpsc::{self, Sender},
};

use crate::{
    utils::{api::get_truck_id_by_vin, avl_packet::AVLPacketToBytes},
    worker::{self, WorkerMessage},
};

use super::records::teltonika_vin_handler::get_truck_vin_from_records;

pub struct TeltonikaConnection<S> {
    teltonika_stream: TeltonikaStream<S>,
    imei: String,
    truck_id: Option<String>,
    truck_vin: Option<String>,
    sender_channel: Sender<WorkerMessage>,
}

impl<S: AsyncWriteExt + AsyncReadExt + Unpin + Sync> TeltonikaConnection<S> {
    /// Creates a new instance of [`TeltonikaConnection`]
    ///
    /// # Arguments
    /// * `stream` - Stream to be passed for [`TeltonikaStream`]. Must implement [`AsyncWriteExt`] and [`AsyncReadExt`]
    /// * `imei` - IMEI of the device
    pub fn new(stream: TeltonikaStream<S>, imei: String) -> Self {
        let (tx, rx) = mpsc::channel::<WorkerMessage>(4000);
        let teltonika_connection = TeltonikaConnection {
            teltonika_stream: stream,
            imei,
            truck_id: None,
            truck_vin: None,
            sender_channel: tx,
        };

        worker::spawn(rx);

        teltonika_connection
    }

    /// Handles the connection with the Teltonika Telematics device
    ///
    /// This function will handle the IMEI of the device and if it is valid, it will run the connection.
    ///
    /// # Arguments
    /// * `stream` - Stream to be passed for [`TeltonikaStream`]. Must implement [`AsyncWriteExt`] and [`AsyncReadExt`]
    /// * `base_file_path` - Base path for the log files
    pub async fn handle_connection(stream: S, base_file_path: &Path) -> Result<(), ()> {
        match Self::handle_imei(TeltonikaStream::new(stream)).await {
            Ok((stream, imei)) => {
                let file_path = base_file_path.join(&imei);
                let mut connection = Self::new(stream, imei);
                connection.run(&file_path).await.expect("Failed to run");
                Ok(())
            }
            Err(_) => Err(()),
        }
    }

    /// Handles the IMEI of the Teltonika Telematics device
    ///
    /// Whether the IMEI is valid, the server will send an approval message to the client.
    ///
    /// # Arguments
    /// * `stream` - Teltonika stream
    async fn handle_imei(mut stream: TeltonikaStream<S>) -> Result<(TeltonikaStream<S>, String), ()> {
        match stream.read_imei_async().await {
            Ok(imei) => {
                info!(target: &imei, "New client connected");
                stream
                    .write_imei_approval_async()
                    .await
                    .expect("Failed to write IMEI approval");
                Ok((stream, imei.to_owned()))
            }
            Err(err) => match err.kind() {
                std::io::ErrorKind::InvalidData => {
                    error!("Failed to parse IMEI from client: {}", err);
                    stream
                        .write_imei_denial_async()
                        .await
                        .expect("Failed to write IMEI denial");
                    Err(())
                }
                _ => {
                    // This is thrown when client connects with empty payload and disconnects immediately after. Performed by health checks and we want to swallow it quietly without bloating the logs.
                    Err(())
                }
            },
        }
    }

    fn log_target(&self) -> &str {
        &self.imei
    }

    /// Runs the connection with the Teltonika Telematics device
    ///
    /// This function will run the connection with the Teltonika Telematics device and handle the incoming frames.
    /// It will also write the data to the log file.
    ///
    /// # Arguments
    /// * `base_log_file_path` - Base path for the log files
    async fn run(&mut self, base_log_file_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let start_of_connection = Utc::now();
        let mut file_handle = self.get_log_file_handle(base_log_file_path);

        loop {
            let start_of_loop = Utc::now();
            if start_of_loop.day() != start_of_connection.day() {
                file_handle = self.get_log_file_handle(base_log_file_path);
            }

            match self.teltonika_stream.read_frame_async().await {
                Ok(frame) => {
                    let records_count = frame.records.len();

                    if let None = self.truck_vin {
                        self.truck_vin = get_truck_vin_from_records(&frame.records);
                    }
                    if self.truck_id.is_none() && self.truck_vin.is_some() {
                        let found_truck_id = get_truck_id_by_vin(&self.truck_vin).await;
                        if found_truck_id.is_some() {
                            debug!(
                                target: self.log_target(),
                                "Found Truck ID [{}] for VIN [{}]",
                                found_truck_id.clone().unwrap(),
                                self.truck_vin.clone().unwrap()
                            );
                            self.truck_id = found_truck_id.map(|id| id.to_string());
                        }
                    }

                    if let Some(vin) = &self.truck_vin {
                        debug!(
                            target: self.log_target(),
                            "Received frame with {} records from VIN [{}]",
                            records_count, vin
                        );
                    } else {
                        debug!(
                            target: self.log_target(),
                            "Received frame with {} records from unknown VIN",
                            records_count
                        );
                    }

                    self.write_data_to_log_file(&mut file_handle, &frame.to_bytes());

                    self.teltonika_stream.write_frame_ack_async(Some(&frame)).await?;

                    if let Err(err) = self
                        .sender_channel
                        .send(WorkerMessage::IncomingFrame {
                            frame,
                            truck_id: self.truck_id.clone(),
                            base_cache_path: base_log_file_path.clone(),
                            imei: self.imei.clone(),
                        })
                        .await
                    {
                        error!(target: self.log_target(), "Failed to send frame to worker: {}", err);
                    };
                }
                Err(err) => match err.kind() {
                    std::io::ErrorKind::ConnectionReset => {
                        info!(target: self.log_target(), "Client disconnected");
                        break;
                    }
                    std::io::ErrorKind::InvalidData => {
                        error!(target: self.log_target(),
                            "Failed to parse frame from client: {}",
                            err
                        );
                    }
                    _ => {
                        error!(target: self.log_target(),
                            "Unknown error when parsing frame from client: {}",
                            err
                        );
                        break;
                    }
                },
            }
        }

        Ok(())
    }

    /// Write data to log file
    ///
    /// # Arguments
    /// * `file_handle` - File handle
    /// * `data` - Data to write to file
    fn write_data_to_log_file(&self, file_handle: &mut Option<File>, data: &[u8]) {
        if cfg!(test) {
            return;
        }
        if let Some(file) = file_handle {
            let encoded = base64::prelude::BASE64_STANDARD.encode(data) + "\\n";
            file.write_all(encoded.as_bytes())
                .expect("Failed to write data to file");
        }
    }

    /// Gets file handle for log file
    ///
    /// # Arguments
    /// * `log_file_path` - Path to log file
    ///
    /// # Returns
    /// * `Option<File>` - File handle
    fn get_log_file_handle(&self, log_file_path: &Path) -> Option<File> {
        if cfg!(not(test)) && log_file_path.file_name().unwrap() != "" {
            let today = Utc::now().format("%Y-%m-%d").to_string();
            create_dir_all(&log_file_path)
                .expect(&format!("Failed to create log file directory `{:#?}`", &log_file_path));
            return Some(
                OpenOptions::new()
                    .read(true)
                    .create(true)
                    .append(true)
                    .open(log_file_path.join(format!("{}.txt", today)))
                    .expect("Failed to open file"),
            );
        }

        return None;
    }
}
