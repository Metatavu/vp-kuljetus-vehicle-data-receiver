use log::info;
use nom_teltonika::AVLFrame;
use testcontainers::{
    core::{logs::consumer::logging_consumer::LoggingConsumer, WaitFor},
    runners::AsyncRunner,
    ContainerAsync, GenericImage, ImageExt,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt, Interest};
use vp_kuljetus_vehicle_data_receiver::utils::avl_packet::AVLPacketToBytes;
use vp_kuljetus_vehicle_data_receiver::utils::imei::build_valid_imei_packet;

/// Image and tag for the data receiver test container
static TEST_APP_IMAGE: &str = "vp-kuljetus-vehicle-data-receiver";
static TEST_APP_TAG: &str = "test";
static FMC234_PORT_NUMBER: u16 = 2340;
static FMC650_PORT_NUMBER: u16 = 6500;

/// Test container for the data receiver service
pub struct DataReceiverTestContainer {
    data_receiver_container: Option<ContainerAsync<GenericImage>>,
}

/// Implementation of DataReceiverTestContainer.
/// This struct provides methods to start the data receiver container and retrieve its host and port information.
/// It allows for testing the data receiver service in isolation.
impl DataReceiverTestContainer {
    pub fn new() -> Self {
        Self {
            data_receiver_container: None,
        }
    }

    /// Starts the data receiver container with the specified configuration.
    /// # Returns
    /// A mutable reference to self, allowing for method chaining.
    /// # Errors
    /// Returns an error if the container fails to start.
    /// # Panics
    /// Panics if the container cannot be started.
    pub async fn start(&mut self) -> &mut Self {
        let data_receiver_container = GenericImage::new(TEST_APP_IMAGE, TEST_APP_TAG)
            .with_wait_for(WaitFor::millis(1000))
            .with_env_var("API_BASE_URL", "http://api-services:8080")
            .with_env_var("VEHICLE_MANAGEMENT_SERVICE_API_KEY", "fake")
            .with_env_var("WRITE_TO_FILE", "true")
            .with_env_var("RUST_LOG", "debug,reqwest=off,hyper=off")
            .with_env_var("PURGE_CHUNK_SIZE", "1000")
            .with_env_var("BASE_FILE_PATH", "/tmp/")
            // .with_log_consumer(LoggingConsumer::new().with_prefix("app"))
            .with_network("tests")
            .with_container_name("data-receiver")
            .with_mapped_port(
                FMC234_PORT_NUMBER,
                testcontainers::core::ContainerPort::Tcp(FMC234_PORT_NUMBER),
            )
            .with_mapped_port(
                FMC650_PORT_NUMBER,
                testcontainers::core::ContainerPort::Tcp(FMC650_PORT_NUMBER),
            );

        self.data_receiver_container = Some(data_receiver_container.start().await.unwrap());

        return self;
    }

    /// Stops the data receiver container.
    pub async fn stop(&mut self) {
        if let Some(container) = self.data_receiver_container.take() {
            container.stop().await.unwrap();
            container.rm().await.unwrap();
        }
    }

    /// Retrieves the host address of the data receiver container.
    /// # Returns
    /// A `String` containing the host address.
    /// # Errors
    /// Returns an error if the host cannot be retrieved.
    /// # Panics
    /// Panics if the host cannot be retrieved.
    pub async fn get_host(&self) -> String {
        return self
            .data_receiver_container
            .as_ref()
            .expect("Data receiver container not started")
            .get_host()
            .await
            .unwrap()
            .to_string();
    }

    /// Returns TCP port number for FMC 650.
    /// # Returns
    /// A `u16` representing the FMC 650 port number.
    /// # Errors
    /// Returns an error if the port cannot be retrieved.
    /// # Panics
    /// Panics if the port cannot be retrieved.
    pub async fn get_fmc650_port(&self) -> u16 {
        return self
            .data_receiver_container
            .as_ref()
            .expect("Data receiver container not started")
            .get_host_port_ipv4(FMC650_PORT_NUMBER)
            .await
            .unwrap();
    }

    /// Returns TCP port number for FMC 234.
    /// # Returns
    /// A `u16` representing the FMC 234 port number.
    /// # Errors
    /// Returns an error if the port cannot be retrieved.
    /// # Panics
    /// Panics if the port cannot be retrieved.
    pub async fn get_fmc234_port(&self) -> u16 {
        return self
            .data_receiver_container
            .as_ref()
            .expect("Data receiver container not started")
            .get_host_port_ipv4(FMC234_PORT_NUMBER)
            .await
            .unwrap();
    }

    /// Sends a IMEI packet to the data receiver container.
    /// # Arguments
    /// * `tcp_stream` - A mutable reference to the TCP stream to send the packet.
    /// * `imei` - A string slice containing the IMEI number to send.
    /// # Returns
    /// A `Result` indicating success or failure.
    /// # Errors
    /// Returns an error if the packet cannot be sent or acknowledged.
    /// # Panics
    /// Panics if the IMEI packet cannot be built or sent.
    pub async fn send_imei_packet(&self, tcp_stream: &mut tokio::net::TcpStream, imei: &str) {
        let imei_packet = build_valid_imei_packet(&imei);
        tcp_stream.write_all(&imei_packet).await.unwrap();
        let mut ack = [0u8; 1];
        tcp_stream.read_exact(&mut ack).await.unwrap();
        assert_eq!(ack[0], 0x01, "server did not ACK with 0x01");
    }

    /// Sends an AVL frame to the data receiver container and verifies the server’s acknowledgment.
    ///
    /// # Arguments
    /// * `tcp_stream` - A mutable reference to the TCP stream to send the AVL frame.
    /// * `avl_frame` - A reference to the AVL frame to send.
    ///
    /// # Returns
    /// * `Ok(())` if the frame was successfully sent and acknowledged by the server.
    /// * `Err(anyhow::Error)` if the send or acknowledgment process fails at any step.
    ///
    /// # Errors
    /// This function returns an error if:
    /// - The AVL frame cannot be serialized to bytes.
    /// - The frame fails to write to the TCP stream.
    /// - The acknowledgment cannot be read from the TCP stream.
    /// - The server returns an incorrect record count.
    ///
    /// # Example
    /// ```rust,ignore
    /// let result = api.send_avl_frame(&mut tcp_stream, &frame).await;
    /// assert!(result.is_ok(), "Expected successful frame send: {:?}", result);
    /// ```
    pub async fn send_avl_frame(
        &self,
        tcp_stream: &mut tokio::net::TcpStream,
        avl_frame: &AVLFrame,
    ) -> anyhow::Result<()> {
        info!("Sending AVL frame with {} records", avl_frame.records.len());

        tcp_stream
            .write_all(&avl_frame.to_bytes())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to write AVL frame: {}", e))?;

        info!("AVL frame sent, waiting for response...");
        let mut buf = [0u8; 4];
        tcp_stream
            .read_exact(&mut buf)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read response: {}", e))?;

        let count_with = u32::from_be_bytes(buf);

        if count_with != avl_frame.records.len() as u32 {
            anyhow::bail!(
                "Server returned incorrect record count: expected {}, got {}",
                avl_frame.records.len(),
                count_with
            );
        }

        Ok(())
    }

    /// Opens a TCP stream to the FMC650 port of the data receiver container.
    ///
    /// Caller must ensure that the connection is closed after use with `tokio::net::TcpStream::shutdown`.
    ///
    /// # Returns
    /// A `tokio::net::TcpStream` connected to the FMC650 port.
    /// # Errors
    /// Returns an error if the connection cannot be established.
    /// # Panics
    /// Panics if the connection fails.
    pub async fn get_tcp_stream_fmc650(&self) -> tokio::net::TcpStream {
        let host = self.get_host().await;
        let port = self.get_fmc650_port().await;
        tokio::net::TcpStream::connect((host.as_str(), port)).await.unwrap()
    }

    /// Opens a TCP stream to the FMC234 port of the data receiver container.
    /// ///
    /// Caller must ensure that the connection is closed after use with `tokio::net::TcpStream::shutdown`.
    /// # Returns
    /// A `tokio::net::TcpStream` connected to the FMC234 port.
    /// # Errors
    /// Returns an error if the connection cannot be established.
    /// # Panics
    /// Panics if the connection fails.
    pub async fn get_tcp_stream_fmc234(&self) -> tokio::net::TcpStream {
        let host = self.get_host().await;
        let port = self.get_fmc234_port().await;
        tokio::net::TcpStream::connect((host.as_str(), port)).await.unwrap()
    }
}
