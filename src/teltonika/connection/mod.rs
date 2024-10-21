use base64::Engine;
use chrono::{Datelike, Utc};
use log::{debug, error, info, warn};
use nom_teltonika::{AVLRecord, TeltonikaStream};
use std::{
    fs::{create_dir_all, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::utils::{
    api::{delete_truck_driver_card_by_id, get_truck_driver_card_id, get_truck_id_by_vin},
    avl_packet::AVLPacketToBytes,
};

use super::records::TeltonikaRecordsHandler;

pub struct TeltonikaConnection<S> {
    teltonika_stream: TeltonikaStream<S>,
    imei: String,
    truck_id: Option<String>,
    truck_vin: Option<String>,
    records_handler: TeltonikaRecordsHandler,
    card_remove_threshold: u16,
    driver_one_card_removed_at: Option<i64>,
}

impl<S: AsyncWriteExt + AsyncReadExt + Unpin> TeltonikaConnection<S> {
    /// Creates a new instance of [`TeltonikaConnection`]
    ///
    /// # Arguments
    /// * `stream` - Stream to be passed for [`TeltonikaStream`]. Must implement [`AsyncWriteExt`] and [`AsyncReadExt`]
    /// * `imei` - IMEI of the device
    /// * `base_file_path` - Base path for the log files
    /// * `card_remove_threshold` - Threshold for removing the driver card
    pub fn new(
        stream: TeltonikaStream<S>,
        imei: String,
        base_file_path: &Path,
        card_remove_threshold: u16,
    ) -> Self {
        TeltonikaConnection {
            teltonika_stream: stream,
            records_handler: TeltonikaRecordsHandler::new(&base_file_path, None, imei.clone()),
            imei,
            truck_id: None,
            truck_vin: None,
            card_remove_threshold,
            driver_one_card_removed_at: None,
        }
    }

    /// Handles the connection with the Teltonika Telematics device
    ///
    /// This function will handle the IMEI of the device and if it is valid, it will run the connection.
    ///
    /// # Arguments
    /// * `stream` - Stream to be passed for [`TeltonikaStream`]. Must implement [`AsyncWriteExt`] and [`AsyncReadExt`]
    /// * `base_file_path` - Base path for the log files
    /// * `card_remove_threshold` - Threshold for removing the driver card
    pub async fn handle_connection(
        stream: S,
        base_file_path: &Path,
        card_remove_threshold: u16,
    ) -> Result<(), ()> {
        match Self::handle_imei(TeltonikaStream::new(stream)).await {
            Ok((stream, imei)) => {
                let file_path = base_file_path.join(&imei);
                let mut connection = Self::new(stream, imei, &file_path, card_remove_threshold);
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
    async fn handle_imei(
        mut stream: TeltonikaStream<S>,
    ) -> Result<(TeltonikaStream<S>, String), ()> {
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

    /// Handles the removal of the driver card
    ///
    /// Teltonika Telematics devices are configured to send eventual records of driver one card presence whenever the value changes.
    /// It is possible that we sometimes receive false events of driver card removal,
    /// so we need to check whether is has actually been removed longer than the [`driver_card_removal_threshold`]
    ///
    /// # Arguments
    /// * `records` - Records to be checked for driver card removal events
    async fn handle_driver_one_card_removal(&mut self, mut records: &mut Vec<AVLRecord>) {
        if let Some((driver_one_card_present_in_frame, timestamp)) = self
            .records_handler
            .get_driver_one_card_presence_from_records(&mut records)
        {
            let now = Utc::now().timestamp_millis();
            if !driver_one_card_present_in_frame {
                let Some(card_removed_at) = self.driver_one_card_removed_at else {
                    self.driver_one_card_removed_at = Some(now);
                    return;
                };
                if now - card_removed_at > self.card_remove_threshold.into() {
                    let Some(truck_id) = &self.truck_id else {
                        warn!(target: self.log_target(), "Attempted to remove driver card from truck with no ID");
                        return;
                    };
                    let Some(driver_card_id) = get_truck_driver_card_id(truck_id.clone()).await
                    else {
                        return;
                    };
                    if let Some(timestamp) = timestamp {
                        delete_truck_driver_card_by_id(truck_id.clone(), driver_card_id, timestamp)
                            .await;
                    }
                }

                return;
            }

            self.driver_one_card_removed_at = None;
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
    async fn run(
        &mut self,
        base_log_file_path: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let start_of_connection = Utc::now();
        let mut file_handle = self.get_log_file_handle(base_log_file_path);

        loop {
            let start_of_loop = Utc::now();
            if start_of_loop.day() != start_of_connection.day() {
                file_handle = self.get_log_file_handle(base_log_file_path);
            }

            match self.teltonika_stream.read_frame_async().await {
                Ok(mut frame) => {
                    let records_count = frame.records.len();
                    self.handle_driver_one_card_removal(&mut frame.records)
                        .await;

                    if let None = self.truck_vin {
                        self.truck_vin = self
                            .records_handler
                            .get_truck_vin_from_records(&frame.records);
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
                            self.records_handler
                                .set_truck_id(found_truck_id.clone().map(|id| id.to_string()));
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

                    self.teltonika_stream
                        .write_frame_ack_async(Some(&frame))
                        .await?;

                    self.records_handler.handle_records(frame.records).await;

                    if let Some(id) = &self.truck_id {
                        info!(target: self.log_target(), "Purging cache for truck ID: [{}]...", id);
                        self.records_handler.purge_cache().await;
                    }
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
            create_dir_all(&log_file_path).expect(&format!(
                "Failed to create log file directory `{:#?}`",
                &log_file_path
            ));
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
