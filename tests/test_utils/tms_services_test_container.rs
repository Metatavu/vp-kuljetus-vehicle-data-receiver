use std::time::Duration;

use serde_json::json;
use testcontainers::{
    core::{logs::consumer::logging_consumer::LoggingConsumer, IntoContainerPort, WaitFor},
    runners::AsyncRunner,
    ContainerAsync, GenericImage, ImageExt,
};
use uuid::Uuid;

use crate::test_utils::wiremock_client::WiremockClient;

/// A mock service for VP TMS API services using Wiremock.
pub struct TmsServicesTestContainer {
    wiremock_container: Option<ContainerAsync<GenericImage>>,
}

/// Implementation of TmsServicesMock.
///
/// This struct provides methods to start a Wiremock container and set up stubs for various API endpoints used in the VP TMS system.
/// It allows for mocking API responses and verifying interactions with the API during tests.
impl TmsServicesTestContainer {
    /// Creates a new instance of TmsServicesTestContainer.
    /// # Returns
    /// A new instance of TmsServicesTestContainer.
    pub fn new() -> Self {
        Self {
            wiremock_container: None,
        }
    }

    /// Starts the Wiremock container with the specified configuration.
    /// # Returns
    /// A reference to the started TmsServicesMock instance.
    /// # Errors
    /// Returns an error if the Wiremock container fails to start.
    pub async fn start(&mut self) -> &mut Self {
        let wiremock_container = GenericImage::new("wiremock/wiremock", "3.13.1")
            .with_exposed_port(8080.tcp())
            .with_wait_for(WaitFor::message_on_stdout("3.13.1"))
            .with_network("tests")
            .with_container_name("api-services")
            .with_log_consumer(LoggingConsumer::new().with_prefix("wiremock"));

        self.wiremock_container = Some(wiremock_container.start().await.unwrap());

        return self;
    }

    /// Mocks the creation of a temperature reading.
    /// This method sets up a stub for the `/v1/temperatureReadings` endpoint
    /// that returns a 200 OK response with an empty JSON body.
    /// # Errors
    /// Returns an error if the stub setup fails.
    pub async fn mock_create_temperature_reading(&self) {
        let wiremock_client = self.get_wiremock_client().await;

        wiremock_client
            .stub("POST", "/v1/temperatureReadings", 200, Some(json!({})), None)
            .await
            .unwrap();
    }

    /// Mocks the retrieval of a trackable by IMEI.
    /// This method sets up a stub for the `/v1/trackables/{imei}` endpoint
    /// that returns a 200 OK response with a JSON object containing the trackable ID,
    /// IMEI, and trackable type.
    /// # Arguments
    /// * `imei` - The IMEI number of the trackable to mock.
    /// # Errors
    /// Returns an error if the stub setup fails.
    pub async fn mock_get_trackable(&self, imei: &str) {
        let wiremock_client = self.get_wiremock_client().await;

        wiremock_client
            .stub(
                "GET",
                format!("/v1/trackables/{}", imei).as_str(),
                200,
                Some(json!({
                    "id": Uuid::new_v4().to_string(),
                    "imei": imei,
                    "trackableType": "TOWABLE"
                })),
                None,
            )
            .await
            .unwrap();
    }

    /// Waits for a specified number of temperature readings to be received.
    /// This method checks the Wiremock server for the number of POST requests made to the `/v1/temperatureReadings` endpoint
    /// and waits until the specified count is reached or the timeout is reached.
    /// # Arguments
    /// * `count` - The number of temperature readings to wait for.
    /// # Returns
    /// The number of temperature readings received.
    /// # Errors
    /// Returns an error if the request to wait for readings fails.
    /// # Panics
    /// Panics if the Wiremock client fails to wait for the specified number of requests.
    pub async fn wait_for_temperature_reading(&self, count: u64) -> u64 {
        let wiremock_client = self.get_wiremock_client().await;
        let reading_count = wiremock_client
            .wait_requests("POST", "/v1/temperatureReadings", count, Duration::from_secs(30))
            .await
            .unwrap();

        return reading_count;
    }

    /// Gets the host and port of the Wiremock container.
    /// # Returns
    /// A tuple containing the host and port of the Wiremock container.
    /// # Errors
    /// Returns an error if the host and port cannot be retrieved.
    async fn get_host_and_port(&self) -> (String, u16) {
        let container = self.wiremock_container.as_ref().expect("Wiremock not started");
        let wiremock_host = container.get_host().await.unwrap().to_string();
        let wiremock_port = container.get_host_port_ipv4(8080).await.unwrap();
        (wiremock_host, wiremock_port)
    }

    /// Gets a Wiremock client configured with the Wiremock host and port.
    /// # Returns
    /// A WiremockClient instance configured with the Wiremock base URL.
    /// # Errors
    /// Returns an error if the Wiremock client cannot be created.
    /// # Panics
    /// Panics if the Wiremock client fails to create due to an invalid URL.
    async fn get_wiremock_client(&self) -> WiremockClient {
        let (wiremock_host, wiremock_port) = self.get_host_and_port().await;
        WiremockClient::new(&format!("http://{}:{}", wiremock_host, wiremock_port)).unwrap()
    }
}
