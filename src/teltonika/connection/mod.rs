use log::{debug, error, info, warn};
use nom_teltonika::TeltonikaStream;
use sqlx::{MySql, Pool};
use std::{
    io::{Error, ErrorKind},
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::mpsc::{self},
    time::timeout,
};
use vehicle_management_service::models::Trackable;

use crate::{
    utils::api::get_trackable,
    worker::{self, Worker, WorkerMessage},
    Listener,
};

pub struct TeltonikaConnection<S> {
    teltonika_stream: TeltonikaStream<S>,
    imei: String,
    trackable: Option<Trackable>,
    worker: Worker,
    listener: Listener,
}

impl<S: AsyncWriteExt + AsyncReadExt + Unpin + Sync> TeltonikaConnection<S> {
    /// Creates a new instance of [`TeltonikaConnection`]
    ///
    /// # Arguments
    /// * `stream` - Stream to be passed for [`TeltonikaStream`]. Must implement [`AsyncWriteExt`] and [`AsyncReadExt`]
    /// * `imei` - IMEI of the device
    /// * `listener` - Listener
    pub fn new(stream: TeltonikaStream<S>, imei: String, listener: Listener, database_pool: Pool<MySql>) -> Self {
        let channel = mpsc::channel::<WorkerMessage>(4000);
        let teltonika_connection = TeltonikaConnection {
            teltonika_stream: stream,
            imei: imei.clone(),
            trackable: None,
            worker: worker::spawn_2(channel, imei, database_pool.clone()),
            listener: listener,
        };

        teltonika_connection
    }

    /// Handles the connection with the Teltonika Telematics device
    ///
    /// This function will handle the IMEI of the device and if it is valid, it will run the connection.
    ///
    /// # Arguments
    /// * `stream` - Stream to be passed for [`TeltonikaStream`]. Must implement [`AsyncWriteExt`] and [`AsyncReadExt`]
    /// * `base_file_path` - Base path for the log files
    /// * `listener` - Listener
    pub async fn handle_connection(stream: S, listener: &Listener, database_pool: Pool<MySql>) -> Result<(), Error> {
        match Self::handle_imei(TeltonikaStream::new(stream)).await {
            Ok((stream, imei)) => {
                let mut connection = Self::new(stream, imei, *listener, database_pool);
                connection.run().await.expect("Failed to run");
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    /// Handles the IMEI of the Teltonika Telematics device
    ///
    /// Whether the IMEI is valid, the server will send an approval message to the client.
    ///
    /// # Arguments
    /// * `stream` - Teltonika stream
    async fn handle_imei(mut stream: TeltonikaStream<S>) -> Result<(TeltonikaStream<S>, String), Error> {
        match stream.read_imei_async().await {
            Ok(imei) => {
                if !imei::valid(&imei) {
                    return Err(Error::new(ErrorKind::ConnectionAborted, "Invalid IMEI"));
                }

                info!(target: &imei, "New client connected");
                stream
                    .write_imei_approval_async()
                    .await
                    .expect("Failed to write IMEI approval");

                return Ok((stream, imei.to_owned()));
            }
            Err(err) => match err.kind() {
                std::io::ErrorKind::InvalidData => {
                    error!("Failed to parse IMEI from client: {}", err);
                    stream
                        .write_imei_denial_async()
                        .await
                        .expect("Failed to write IMEI denial");

                    return Err(err);
                }
                _ => {
                    // This is thrown when client connects with empty payload and disconnects immediately after. Performed by health checks and we want to swallow it quietly without bloating the logs.
                    return Err(err);
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
    async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            if self.trackable.is_none() {
                self.trackable = get_trackable(&self.imei).await;
            }

            match self.teltonika_stream.read_frame_async().await {
                Ok(frame) => {
                    let records_count = frame.records.len();

                    debug!(
                        target: self.log_target(),
                        "Received frame with {records_count} records from"

                    );

                    let ack_result = timeout(
                        Duration::from_secs(60),
                        self.teltonika_stream.write_frame_ack_async(Some(&frame)),
                    )
                    .await;

                    match ack_result {
                        Ok(Ok(())) => debug!(target: self.log_target(),"ACK sent successfully"),
                        Ok(Err(e)) => error!(target: self.log_target(),"ACK write failed: {}", e),
                        Err(_) => warn!(target: self.log_target(),"ACK write timed out"),
                    }

                    let send_result = self
                        .worker
                        .send(WorkerMessage::IncomingFrame {
                            frame: frame.clone(),
                            trackable: self.trackable.clone(),
                            imei: self.imei.clone(),
                            listener: self.listener,
                        })
                        .await;

                    match send_result {
                        Ok(_) => debug!(target: self.log_target(), "Frame sent to worker successfully"),
                        Err(err) => error!(target: self.log_target(), "Failed to send frame to worker: {}", err),
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

                        // If the frame is invalid, we send an zero response to the client,
                        // to indicate that the frame was not processed and need to be sent again.
                        let teltonika_inner_stream = &mut self.teltonika_stream.inner_mut();
                        teltonika_inner_stream.write_i32(0).await.unwrap();
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
}
