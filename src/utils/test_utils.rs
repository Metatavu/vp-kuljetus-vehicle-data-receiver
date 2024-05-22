use std::str::FromStr;

use httpmock::{
    Method::{GET, POST},
    MockServer, Regex,
};
use nom_teltonika::AVLEventIO;
use tempfile::tempdir;
use uuid::Uuid;
use vehicle_management_service::models::{PublicTruck, TruckDriverCard};

use crate::teltonika_handler::teltonika_records_handler::TeltonikaRecordsHandler;

/// Converts a driver card ID to two part events.
///
/// This function is a reverse implementation of what's described in [Teltonika Documentation](https://wiki.teltonika-gps.com/view/DriverID)
/// where the driver card part is converted to a hexadecimal number from an ASCII-string.
pub fn driver_card_id_to_two_part_events(driver_card_id: String) -> [AVLEventIO; 2] {
    let (driver_card_id_msb, driver_card_id_lsb) = split_at_half(driver_card_id);
    let driver_card_id_msb_dec = driver_card_part_to_dec(&driver_card_id_msb);
    let driver_card_id_lsb_dec = driver_card_part_to_dec(&driver_card_id_lsb);
    let driver_card_id_msb_event = AVLEventIO {
        id: 195,
        value: nom_teltonika::AVLEventIOValue::U64(driver_card_id_msb_dec),
    };
    let driver_card_id_lsb_event = AVLEventIO {
        id: 196,
        value: nom_teltonika::AVLEventIOValue::U64(driver_card_id_lsb_dec),
    };
    return [driver_card_id_msb_event, driver_card_id_lsb_event];
}

/// Splits a String at half
pub fn split_at_half(string: String) -> (String, String) {
    let half = string.len() / 2;
    let (part_1, part_2) = string.split_at(half);

    return (part_1.to_string(), part_2.to_string());
}

/// Converts a string to a hexadecimal string
pub fn string_to_hex_string(string: &str) -> String {
    return string
        .as_bytes()
        .iter()
        .map(|byte| format!("{:02X}", byte))
        .collect::<Vec<String>>()
        .concat();
}

/// Reverses a string slice
///
/// This function is not used in the implementation at the moment but is kept in case it is needed later.
#[allow(dead_code)]
pub fn reverse_str(string: &str) -> String {
    return string.chars().rev().collect::<String>();
}

/// Converts a driver card part to a decimal number
pub fn driver_card_part_to_dec(driver_card_part: &str) -> u64 {
    let driver_card_part_hex = string_to_hex_string(driver_card_part);

    return u64::from_str_radix(&driver_card_part_hex, 16).unwrap();
}

/// Reads IMEI from the buffer
///
/// # Arguments
/// * `buffer` - Buffer for reading data from socket
///
/// # Returns
/// * `(bool, Option<String>)` - Whether the IMEI was successfully parsed and the IMEI itself as an `Option<String>`
pub fn read_imei(buffer: &Vec<u8>) -> (bool, Option<String>) {
    let result = nom_teltonika::parser::imei(&buffer);
    match result {
        Ok((_, imei)) => (true, Some(imei)),
        Err(_) => (false, None),
    }
}

/// Gets a TeltonikaRecordsHandler for testing
///
/// Uses a temporary directory for the cache
pub fn get_teltonika_records_handler(truck_id: Option<String>) -> TeltonikaRecordsHandler {
    let test_cache_dir = tempdir().unwrap();
    let test_cache_path = test_cache_dir.path();

    return TeltonikaRecordsHandler::new(test_cache_path, truck_id);
}

/// Starts a mock server for the Vehicle Management Service
pub fn start_vehicle_management_mock() {
    let mock_server = MockServer::start();
    let mut server_address = String::from("http://");
    server_address.push_str(mock_server.address().to_string().as_str());

    std::env::set_var("API_BASE_URL", &server_address);
    std::env::set_var("VEHICLE_MANAGEMENT_SERVICE_API_KEY", "API_KEY");

    let _public_trucks_mock = mock_server.mock(|when, then| {
        when.method(GET)
            .path("/v1/publicTrucks")
            .header("X-API-KEY", "API_KEY");
        then.status(200)
            .header("Content-Type", "application/json")
            .json_body_obj(&[PublicTruck {
                id: Some(Uuid::from_str("3FFAF18C-69E4-4F8A-9179-9AEC5BC96E1C").unwrap()),
                name: Some(String::from("1")),
                plate_number: String::from("ABC-123"),
                vin: String::from("W1T96302X10704959"),
            }]);
    });

    let _create_truck_speed_mock = mock_server.mock(|when, then| {
        when.method(POST)
            .path_matches(Regex::new(r"/v1/trucks/.{36}/speeds").unwrap())
            .header("X-API-KEY", "API_KEY");
        then.status(201);
    });
    let _create_truck_driver_card_mock = mock_server.mock(|when, then| {
        when.method(POST)
            .path_matches(Regex::new(r"/v1/trucks/.{36}/driverCards").unwrap())
            .header("X-API-KEY", "API_KEY");
        then.status(201)
            .header("Content-Type", "application/json")
            .json_body_obj(&TruckDriverCard { id: String::new() });
    });
    let _create_truck_drive_state_mock = mock_server.mock(|when, then| {
        when.method(POST)
            .path_matches(Regex::new(r"/v1/trucks/.{36}/driveState").unwrap())
            .header("X-API-KEY", "API_KEY");
        then.status(201);
    });
}
