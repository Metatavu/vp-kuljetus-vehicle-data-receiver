#![allow(dead_code)]
#[cfg(test)]
pub mod avl_record_builder;
#[cfg(test)]
pub mod avl_frame_builder;
#[cfg(test)]
pub mod imei;
#[cfg(test)]
pub mod avl_packet;

#[cfg(test)]
pub mod utilities {
    use httpmock::{Method::{GET, POST}, MockServer, Regex};
    use tempfile::tempdir;
    use vehicle_management_service_client::model::PublicTruck;

    use crate::teltonika_handler::teltonika_records_handler::TeltonikaRecordsHandler;

    /// Converts a hex string to a byte vector
    ///
    /// # Arguments
    /// * `input` - The hex string to convert
    ///
    /// # Returns
    /// * `Vec<u8>` - The byte vector
    pub fn str_to_bytes(input: &str) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![];
        for (i, char) in input.chars().enumerate() {
            let val = if i % 2 != 0 {
                format!("{}{}", input.chars().nth(i - 1).unwrap(), char)
            } else {
                continue;
            };
            bytes.push(u8::from_str_radix(&val, 16).unwrap())
        }

        return bytes;
    }

    /// Gets a TeltonikaRecordsHandler for testing
    ///
    /// Uses a temporary directory for the cache
    pub fn get_teltonika_records_handler(truck_id: Option<String>) -> TeltonikaRecordsHandler {
        let test_cache_dir = tempdir().unwrap();
        let test_cache_path = test_cache_dir.path();

        return TeltonikaRecordsHandler::new( test_cache_path, truck_id);
    }

    /// Starts a mock server for the Vehicle Management Service
    pub fn start_vehicle_management_mock() {
        let mock_server = MockServer::start();
        let mut server_address = String::from("http://");
        server_address.push_str(mock_server.address().to_string().as_str());

        std::env::set_var("VEHICLE_MANAGEMENT_SERVICE_CLIENT_BASE_URL", &server_address);
        std::env::set_var("VEHICLE_MANAGEMENT_SERVICE_API_KEY", "API_KEY");

        let _public_trucks_mock = mock_server.mock(|when, then| {
            when.method(GET)
                .path("/vehicle-management/v1/publicTrucks")
                .header("X-API-KEY", "API_KEY");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body_obj(&[PublicTruck{
                    id: Some(String::from("3FFAF18C-69E4-4F8A-9179-9AEC5BC96E1C")),
                    name: Some(String::from("1")),
                    plate_number: String::from("ABC-123"),
                    vin: String::from("W1T96302X10704959"),
                }]);
        });

        let _create_truck_speed_mock = mock_server.mock(|when, then| {
            when.method(POST)
                .path_matches(Regex::new(r"/vehicle-management/v1/trucks/.{36}/speeds").unwrap())
                .header("X-API-KEY", "API_KEY");
            then.status(201)
                .header("Content-Type", "application/json")
                .json_body_obj(&());
        });

        let _create_truck_locations_mock = mock_server.mock(|when, then| {
            when.method(POST)
                .path_matches(Regex::new(r"/vehicle-management/v1/trucks/.{36}/locations").unwrap())
                .header("X-API-KEY", "API_KEY");
            then.status(201)
                .header("Content-Type", "application/json")
                .json_body_obj(&());
        });
    }
}