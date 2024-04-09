use std::{fs::{create_dir_all, File, OpenOptions}, io::Write, path::Path};
use base64::Engine;
use chrono::{Datelike, Utc};
use log::{debug, error, info};
use nom_teltonika::TeltonikaStream;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::teltonika_handler::TeltonikaRecordsHandler;
use crate::vehicle_management_service::VehicleManagementService;
use crate::utils::avl_packet::AVLPacketToBytes;

pub struct TeltonikaConnection {
  teltonika_stream: TeltonikaStream<TcpStream>,
  imei: Option<String>,
}

impl TeltonikaConnection {
  /// Creates a new instance of [`TeltonikaConnection`]
  pub fn new(tcp_stream: TcpStream) -> Self {
    TeltonikaConnection {
      teltonika_stream: TeltonikaStream::new(tcp_stream),
      imei: None,
    }
  }

  /// Handles the connection with the Teltonika Telematics device
  ///
  /// This function will handle the IMEI of the device and if it is valid, it will run the connection.
  ///
  /// # Arguments
  /// * `base_file_path` - Base path for the log files
  pub async fn handle_connection(&mut self, base_file_path: String) -> Result<(), ()> {
    match self.handle_imei().await {
      Ok(_) => {
        self.run(base_file_path).await.expect("Failed to run");
        Ok(())
      },
      Err(_) => {
        Err(())
      }
    }
  }

  /// Handles the IMEI of the Teltonika Telematics device
  ///
  /// Whether the IMEI is valid, the server will send an approval message to the client.
  async fn handle_imei(&mut self) -> Result<(), ()> {
    match &self.teltonika_stream.read_imei_async().await {
      Ok(imei) => {
        info!("New client connected with IMEI [{}]", imei);
        self.teltonika_stream.write_imei_approval_async().await.expect("Failed to write IMEI approval");
        self.imei = Some(imei.to_owned());
        Ok(())
      },
      Err(err) => {
        error!("Failed to parse IMEI from client: {}", err);
        self.teltonika_stream.write_imei_denial_async().await.expect("Failed to write IMEI denial");
        self.get_socket().shutdown().await.expect("Failed to shutdown socket");
        Err(())
      }
    }
  }

  /// Runs the connection with the Teltonika Telematics device
  ///
  /// This function will run the connection with the Teltonika Telematics device and handle the incoming frames.
  /// It will also write the data to the log file.
  ///
  /// # Arguments
  /// * `base_file_path` - Base path for the log files
  async fn run(&mut self, base_file_path: String) -> Result<(), Box<dyn std::error::Error>> {
    let imei = self.imei.to_owned().unwrap();
    let file_path = Path::new(&base_file_path).join(&imei);
    let start_of_connection = Utc::now();

    let mut file_handle = self.get_log_file_handle(&file_path);
    let mut truck_vin: Option<String> = None;
    let mut truck_id: Option<String> = None;
    let mut teltonika_records_handler = TeltonikaRecordsHandler::new(&file_path, truck_id.clone());

    loop {
      let start_of_loop = Utc::now();
      if start_of_loop.day() != start_of_connection.day() {
        file_handle = self.get_log_file_handle(&file_path);
      }

      match self.teltonika_stream.read_frame_async().await {
        Ok(frame) => {
          let records_count = frame.records.len();

          if let None = truck_vin {
            truck_vin = teltonika_records_handler.get_truck_vin_from_records(&frame.records);
          }

          if truck_id.is_none() && truck_vin.is_some() {
            truck_id = VehicleManagementService::get_truck_id_by_vin(&truck_vin).await;
            debug!("Found Truck ID [{}] for VIN [{}]", truck_id.clone().unwrap(), truck_vin.clone().unwrap());
            teltonika_records_handler.set_truck_id(truck_id.clone());
          }

          if let Some(vin) = &truck_vin {
            debug!("Received frame with {} records from VIN [{}] with IMEI [{}]", records_count, vin, imei);
          } else {
            debug!("Received frame with {} records from unknown VIN with IMEI [{}]", records_count, imei);
          }

          self.write_data_to_log_file(&mut file_handle, &frame.to_bytes());

          self.teltonika_stream.write_frame_ack_async(Some(&frame)).await?;
        },
        Err(_) => {
          info!("Client with IMEI [{}] disconnected", self.imei.as_ref().unwrap());
          break;
      }
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
          file.write_all(encoded.as_bytes()).expect("Failed to write data to file");
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
          create_dir_all(&log_file_path).expect(&format!("Failed to create log file directory `{:#?}`", &log_file_path));
          return Some(
              OpenOptions::new()
                  .read(true)
                  .create(true)
                  .append(true)
                  .open(
                      log_file_path.join(format!("{}.txt", today))
                  )
                  .expect("Failed to open file")
          );
      }

      return None;
  }

  /// Gets the underlying socket of the [`TeltonikaStream`]
  fn get_socket(&mut self) -> &mut TcpStream {
    self.teltonika_stream.inner_mut()
  }
}