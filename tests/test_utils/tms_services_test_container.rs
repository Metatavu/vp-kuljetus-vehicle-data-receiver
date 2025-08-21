use std::time::Duration;

use chrono::Utc;
use log::info;
use serde_json::json;
use testcontainers::{
    core::{logs::consumer::logging_consumer::LoggingConsumer, IntoContainerPort, WaitFor},
    runners::AsyncRunner,
    ContainerAsync, GenericImage, ImageExt,
};
use uuid::Uuid;
use vehicle_management_service::models::trackable;

use crate::test_utils::wiremock_client::WiremockClient;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

/// A mock service for VP TMS API services using Wiremock.
pub struct TmsServicesTestContainer {
    wiremock_container: Option<ContainerAsync<GenericImage>>,
    temperature_reading_mapping_id: Option<String>,
    drive_state_mapping_id: Option<String>,
    driver_card_mapping_id: Option<String>,
    truck_location_mapping_id: Option<String>,
    odometer_reading_mapping_id: Option<String>,
    speed_mapping_id: Option<String>,
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
            temperature_reading_mapping_id: None,
            drive_state_mapping_id: None,
            driver_card_mapping_id: None,
            truck_location_mapping_id: None,
            odometer_reading_mapping_id: None,
            speed_mapping_id: None,
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

    /// Stops the Wiremock container.
    pub async fn stop(&mut self) {
        if let Some(container) = self.wiremock_container.take() {
            container.stop().await.unwrap();
            container.rm().await.unwrap();
        }
    }

    /// Mocks the creation of a temperature reading.
    /// This method sets up a stub for the `/v1/temperatureReadings` endpoint
    /// that returns a response with given code and an empty JSON body.
    ///
    /// If the method `mock_create_temperature_reading` is called multiple times,
    /// it will replace the previous stub with the new one.
    ///
    /// # Arguments
    /// * `status` - The HTTP status code to return for the stubbed request.
    /// # Errors
    /// Returns an error if the stub setup fails.
    /// # Panics
    /// Panics if the Wiremock client fails to create the stub.
    pub async fn mock_create_temperature_reading(&mut self, status: u16) {
        let wiremock_client = self.get_wiremock_client().await;

        if self.temperature_reading_mapping_id.is_some() {
            wiremock_client
                .reset_mapping(self.temperature_reading_mapping_id.as_ref().unwrap())
                .await
                .unwrap();
        }

        self.temperature_reading_mapping_id = Some(
            wiremock_client
                .stub("POST", "/v1/temperatureReadings", status, Some(json!({})), None)
                .await
                .unwrap(),
        );
    }

    /// Mocks drive state creation endpoint
    ///
    /// This method sets up a stub for the `/v1/trucks/{truckId}/driveStates` endpoint
    /// that returns a response with given code and an empty JSON body.
    ///
    /// If the method `mock_create_drive_state` is called multiple times,
    /// it will replace the previous stub with the new one.
    ///
    /// # Arguments
    /// * `status` - The HTTP status code to return for the stubbed request.
    /// # Errors
    /// Returns an error if the stub setup fails.
    /// # Panics
    /// Panics if the Wiremock client fails to create the stub.
    pub async fn mock_create_drive_state(&mut self, truck_id: String, status: u16) {
        let wiremock_client = self.get_wiremock_client().await;

        if self.drive_state_mapping_id.is_some() {
            wiremock_client
                .reset_mapping(self.drive_state_mapping_id.as_ref().unwrap())
                .await
                .unwrap();
        }

        self.drive_state_mapping_id = Some(
            wiremock_client
                .stub(
                    "POST",
                    format!("/v1/trucks/{}/driveStates", truck_id.as_str()).as_str(),
                    status,
                    Some(json!({})),
                    None,
                )
                .await
                .unwrap(),
        );
    }

    /// Mocks the creation of a driver card.
    /// This method sets up a stub for the `/v1/trucks/{truckId}/driverCards` endpoint
    /// that returns a response with given code and an empty JSON body.
    ///
    /// If the method `mock_create_driver_card` is called multiple times,
    /// it will replace the previous stub with the new one.
    ///
    /// # Arguments
    /// * `status` - The HTTP status code to return for the stubbed request.
    /// # Errors
    /// Returns an error if the stub setup fails.
    /// # Panics
    /// Panics if the Wiremock client fails to create the stub.
    pub async fn mock_create_driver_card(&mut self, truck_id: String, status: u16) {
        let wiremock_client = self.get_wiremock_client().await;

        let body = if status == 200 {
            json!({
                "id": Uuid::new_v4().to_string(),
                "timestamp": Utc::now().timestamp_millis(),
            })
        } else {
            json!({})
        };

        if self.driver_card_mapping_id.is_some() {
            wiremock_client
                .reset_mapping(self.driver_card_mapping_id.as_ref().unwrap())
                .await
                .unwrap();
        }

        self.driver_card_mapping_id = Some(
            wiremock_client
                .stub(
                    "POST",
                    format!("/v1/trucks/{}/driverCards", truck_id.as_str()).as_str(),
                    status,
                    Some(body),
                    None,
                )
                .await
                .unwrap(),
        );
    }

    /// Mocks the creation of an odometer reading.
    /// This method sets up a stub for the `/v1/trucks/{truckId}/odometerReadings` endpoint
    /// that returns a response with given code and an empty JSON body.
    ///
    /// If the method `mock_create_odometer_reading` is called multiple times,
    /// it will replace the previous stub with the new one.
    ///
    /// # Arguments
    /// * `status` - The HTTP status code to return for the stubbed request.
    /// # Errors
    /// Returns an error if the stub setup fails.
    /// # Panics
    /// Panics if the Wiremock client fails to create the stub.
    pub async fn mock_create_odometer_reading(&mut self, truck_id: String, status: u16) {
        let wiremock_client = self.get_wiremock_client().await;

        if self.odometer_reading_mapping_id.is_some() {
            wiremock_client
                .reset_mapping(self.odometer_reading_mapping_id.as_ref().unwrap())
                .await
                .unwrap();
        }

        self.odometer_reading_mapping_id = Some(
            wiremock_client
                .stub(
                    "POST",
                    format!("/v1/trucks/{}/odometerReadings", truck_id.as_str()).as_str(),
                    status,
                    Some(json!({})),
                    None,
                )
                .await
                .unwrap(),
        );
    }

    /// Mocks the creation of a speed reading.
    /// This method sets up a stub for the `/v1/trucks/{truckId}/speedReadings` endpoint
    /// that returns a response with given code and an empty JSON body.
    ///
    /// If the method `mock_create_speed` is called multiple times,
    /// it will replace the previous stub with the new one.
    ///
    /// # Arguments
    /// * `status` - The HTTP status code to return for the stubbed request.
    /// # Errors
    /// Returns an error if the stub setup fails.
    /// # Panics
    /// Panics if the Wiremock client fails to create the stub.
    pub async fn mock_create_speed(&mut self, truck_id: String, status: u16) {
        let wiremock_client = self.get_wiremock_client().await;

        if self.speed_mapping_id.is_some() {
            wiremock_client
                .reset_mapping(self.speed_mapping_id.as_ref().unwrap())
                .await
                .unwrap();
        }

        self.speed_mapping_id = Some(
            wiremock_client
                .stub(
                    "POST",
                    format!("/v1/trucks/{}/speeds", truck_id.as_str()).as_str(),
                    status,
                    Some(json!({})),
                    None,
                )
                .await
                .unwrap(),
        );
    }

    /// Mocks truck location create endpoint
    /// This method sets up a stub for the `/v1/trucks/{truckId}/locations` endpoint
    /// that returns a response with given code and an empty JSON body.
    ///
    /// If the method `mock_create_truck_location` is called multiple times,
    /// it will replace the previous stub with the new one.
    ///
    /// # Arguments
    /// * `status` - The HTTP status code to return for the stubbed request.
    /// # Errors
    /// Returns an error if the stub setup fails.
    /// # Panics
    /// Panics if the Wiremock client fails to create the stub.
    pub async fn mock_create_truck_location(&mut self, truck_id: String, status: u16) {
        let wiremock_client = self.get_wiremock_client().await;

        if self.truck_location_mapping_id.is_some() {
            wiremock_client
                .reset_mapping(self.truck_location_mapping_id.as_ref().unwrap())
                .await
                .unwrap();
        }

        self.truck_location_mapping_id = Some(
            wiremock_client
                .stub(
                    "POST",
                    format!("/v1/trucks/{}/locations", truck_id.as_str()).as_str(),
                    status,
                    Some(json!({})),
                    None,
                )
                .await
                .unwrap(),
        );
    }

    /// Resets the all request counts in Wiremock.
    pub async fn reset_counts(&self) {
        let wiremock_client = self.get_wiremock_client().await;
        wiremock_client.reset_counts().await.unwrap();
    }

    /// Mocks the retrieval of a trackable by IMEI.
    /// This method sets up a stub for the `/v1/trackables/{imei}` endpoint
    /// that returns a 200 OK response with a JSON object containing the trackable ID,
    /// IMEI, and trackable type.
    /// # Arguments
    /// * `imei` - The IMEI number of the trackable to mock.
    /// # Errors
    /// Returns an error if the stub setup fails.
    pub async fn mock_get_trackable(&self, imei: &str, trackable_id: &str, trackable_type: &str) {
        let wiremock_client = self.get_wiremock_client().await;

        wiremock_client
            .stub(
                "GET",
                format!("/v1/trackables/{}", imei).as_str(),
                200,
                Some(json!({
                    "id": trackable_id,
                    "imei": imei,
                    "trackableType": trackable_type
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
            .wait_requests("POST", "/v1/temperatureReadings", count, DEFAULT_TIMEOUT)
            .await
            .unwrap();

        return reading_count;
    }

    /// Waits for a specified number of drive state creation requets received.
    ///
    /// # Arguments
    /// * `count` - The number of drive state creation requests to wait for.
    /// * `truck_id` - The ID of the truck for which to wait for drive state creation requests.
    /// # Returns
    /// The number of drive state creation requests received.
    /// # Panics
    /// Panics if the Wiremock client fails to wait for the specified number of requests.
    pub async fn wait_for_drive_state_creation(&self, count: u64, truck_id: &str) -> u64 {
        let wiremock_client = self.get_wiremock_client().await;
        let reading_count = wiremock_client
            .wait_requests(
                "POST",
                format!("/v1/trucks/{}/driveStates", truck_id).as_str(),
                count,
                DEFAULT_TIMEOUT,
            )
            .await
            .unwrap();

        return reading_count;
    }

    /// Waits for a specified number of driver card creation requests received.
    /// This method checks the Wiremock server for the number of POST requests made to the `/v1/trucks/{truckId}/driverCards` endpoint
    /// and waits until the specified count is reached or the timeout is reached.
    /// # Arguments
    /// * `count` - The number of driver card creation requests to wait for.
    /// * `truck_id` - The ID of the truck for which to wait for driver card creation requests.
    /// # Returns
    /// The number of driver card creation requests received.
    /// # Errors
    /// Returns an error if the request to wait for driver card creation fails.
    /// # Panics
    /// Panics if the Wiremock client fails to wait for the specified number of requests.
    pub async fn wait_for_driver_card_creation(&self, count: u64, truck_id: &str) -> u64 {
        let wiremock_client = self.get_wiremock_client().await;
        let reading_count = wiremock_client
            .wait_requests(
                "POST",
                format!("/v1/trucks/{}/driverCards", truck_id).as_str(),
                count,
                DEFAULT_TIMEOUT,
            )
            .await
            .unwrap();

        return reading_count;
    }

    /// Waits for a specified number of odometer readings to be received.
    /// This method checks the Wiremock server for the number of POST requests made to the `/v1/trucks/{truckId}/odometerReadings` endpoint
    /// and waits until the specified count is reached or the timeout is reached.
    /// # Arguments
    /// * `count` - The number of odometer readings to wait for.
    /// * `truck_id` - The ID of the truck for which to wait for odometer readings.
    /// # Returns
    /// The number of odometer readings received.
    /// # Errors
    /// Returns an error if the request to wait for readings fails.
    /// # Panics
    /// Panics if the Wiremock client fails to wait for the specified number of requests.
    pub async fn wait_for_odometer_reading(&self, count: u64, truck_id: &str) -> u64 {
        let wiremock_client = self.get_wiremock_client().await;
        let reading_count = wiremock_client
            .wait_requests(
                "POST",
                format!("/v1/trucks/{}/odometerReadings", truck_id).as_str(),
                count,
                DEFAULT_TIMEOUT,
            )
            .await
            .unwrap();

        return reading_count;
    }

    /// Waits for a specified number of speed readings to be received.
    /// This method checks the Wiremock server for the number of POST requests made to the `/v1/trucks/{truckId}/speedReadings` endpoint
    /// and waits until the specified count is reached or the timeout is reached.
    /// # Arguments
    /// * `count` - The number of speed readings to wait for.
    /// * `truck_id` - The ID of the truck for which to wait for speed readings.
    /// # Returns
    /// The number of speed readings received.
    /// # Errors
    /// Returns an error if the request to wait for readings fails.
    /// # Panics
    /// Panics if the Wiremock client fails to wait for the specified number of requests.
    pub async fn wait_for_speed(&self, count: u64, truck_id: &str) -> u64 {
        let wiremock_client = self.get_wiremock_client().await;
        let reading_count = wiremock_client
            .wait_requests(
                "POST",
                format!("/v1/trucks/{}/speeds", truck_id).as_str(),
                count,
                DEFAULT_TIMEOUT,
            )
            .await
            .unwrap();

        return reading_count;
    }

    /// Waits for a specified number of location readings to be received.
    /// This method checks the Wiremock server for the number of POST requests made to the `/v1/trucks/{truckId}/locations` endpoint
    /// and waits until the specified count is reached or the timeout is reached.
    /// # Arguments
    /// * `count` - The number of location readings to wait for.
    /// * `truck_id` - The ID of the truck for which to wait for location readings.
    /// # Returns
    /// The number of location readings received.
    /// # Errors
    /// Returns an error if the request to wait for readings fails.
    /// # Panics
    /// Panics if the Wiremock client fails to wait for the specified number of requests.
    pub async fn wait_for_location(&self, count: u64, truck_id: &str) -> u64 {
        let wiremock_client = self.get_wiremock_client().await;
        let reading_count = wiremock_client
            .wait_requests(
                "POST",
                format!("/v1/trucks/{}/locations", truck_id).as_str(),
                count,
                DEFAULT_TIMEOUT,
            )
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
