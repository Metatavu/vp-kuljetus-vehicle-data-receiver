use std::str::FromStr;

use httpmock::{
    Method::{DELETE, GET, POST},
    Mock, MockServer, Regex,
};
use nom_teltonika::AVLEventIO;
use tempfile::tempdir;
use uuid::Uuid;
use vehicle_management_service::models::{PublicTruck, TruckDriverCard};

use crate::teltonika::records::TeltonikaRecordsHandler;

/// Converts a VIN number to 3 part events.
pub fn vin_to_three_part_events(vin: String) -> [AVLEventIO; 3] {
    let (first_part, second_part) = vin.split_at(8);
    let (second_part, third_part) = second_part.split_at(8);
    let first_part = string_to_hex_to_dec(first_part);
    let second_part = string_to_hex_to_dec(second_part);
    let third_part = string_to_hex_to_dec(third_part);

    return [
        AVLEventIO {
            id: 233,
            value: nom_teltonika::AVLEventIOValue::U64(first_part),
        },
        AVLEventIO {
            id: 234,
            value: nom_teltonika::AVLEventIOValue::U64(second_part),
        },
        AVLEventIO {
            id: 235,
            value: nom_teltonika::AVLEventIOValue::U8(third_part as u8),
        },
    ];
}

/// Converts a driver card ID to two part events.
///
/// This function is a reverse implementation of what's described in [Teltonika Documentation](https://wiki.teltonika-gps.com/view/DriverID)
/// where the driver card part is converted to a hexadecimal number from an ASCII-string.
pub fn driver_card_id_to_two_part_events(driver_card_id: String) -> [AVLEventIO; 2] {
    let (driver_card_id_msb, driver_card_id_lsb) = split_at_half(driver_card_id);
    let driver_card_id_msb_dec = string_to_hex_to_dec(&driver_card_id_msb);
    let driver_card_id_lsb_dec = string_to_hex_to_dec(&driver_card_id_lsb);
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
pub fn string_to_hex_to_dec(string: &str) -> u64 {
    let driver_card_part_hex = string_to_hex_string(string);

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
/// Uses a temporary directory for the cache.
///
/// # Arguments
/// * `truck_id` - Truck ID to be used for the handler
/// * `imei` - IMEI to be used for the handler
///
/// # Returns
/// * `TeltonikaRecordsHandler` - TeltonikaRecordsHandler instance
pub fn get_teltonika_records_handler(
    truck_id: Option<String>,
    imei: Option<String>,
) -> TeltonikaRecordsHandler {
    let test_cache_dir = tempdir().unwrap();
    let test_cache_path = test_cache_dir.path().to_path_buf();
    let imei = imei.unwrap_or(String::new());

    return TeltonikaRecordsHandler::new(imei, truck_id, test_cache_path);
}

pub fn mock_server() -> MockServer {
    let mock_server = MockServer::start();
    let mut server_address = String::from("http://");
    server_address.push_str(mock_server.address().to_string().as_str());

    std::env::set_var("API_BASE_URL", &server_address);
    std::env::set_var("VEHICLE_MANAGEMENT_SERVICE_API_KEY", "API_KEY");

    mock_server
}

pub trait MockServerExt {
    fn start_all_mocks(&self) -> [Mock; 7] {
        [
            self.public_trucks_mock(),
            self.create_truck_speed_mock(),
            self.create_truck_locations_mock(),
            self.create_truck_driver_card_mock(),
            self.create_truck_drive_state_mock(),
            self.list_driver_cards_mock(),
            self.delete_driver_card_mock(),
        ]
    }
    fn public_trucks_mock(&self) -> Mock;
    fn create_truck_speed_mock(&self) -> Mock;
    fn create_truck_locations_mock(&self) -> Mock;
    fn create_truck_driver_card_mock(&self) -> Mock;
    fn create_truck_drive_state_mock(&self) -> Mock;
    fn list_driver_cards_mock(&self) -> Mock;
    fn delete_driver_card_mock(&self) -> Mock;
}
impl MockServerExt for MockServer {
    fn public_trucks_mock(&self) -> Mock {
        self.mock(|when, then| {
            when.method(GET)
                .path("/v1/publicTrucks")
                .header("X-API-KEY", "API_KEY");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body_obj(&[PublicTruck {
                    id: Some(Uuid::from_str("3ffaf18c-69e4-4f8a-9179-9aec5bc96e1c").unwrap()),
                    name: Some(String::from("1")),
                    plate_number: String::from("ABC-123"),
                    vin: String::from("W1T96302X10704959"),
                }]);
        })
    }
    fn create_truck_speed_mock(&self) -> Mock {
        self.mock(|when, then| {
            when.method(POST)
                .path_matches(Regex::new(r"/v1/trucks/.{36}/speeds").unwrap())
                .header("X-API-KEY", "API_KEY");
            then.status(201);
        })
    }
    fn create_truck_locations_mock(&self) -> Mock {
        self.mock(|when, then| {
            when.method(POST)
                .path_matches(Regex::new(r"/v1/trucks/.{36}/locations").unwrap())
                .header("X-API-KEY", "API_KEY");
            then.status(201);
        })
    }
    fn create_truck_driver_card_mock(&self) -> Mock {
        self.mock(|when, then| {
            when.method(POST)
                .path_matches(Regex::new(r"^/v1/trucks/.{36}/driverCards$").unwrap())
                .header("X-API-KEY", "API_KEY");
            then.status(201)
                .header("Content-Type", "application/json")
                .json_body_obj(&TruckDriverCard {
                    id: String::new(),
                    timestamp: chrono::Utc::now().timestamp(),
                });
        })
    }
    fn create_truck_drive_state_mock(&self) -> Mock {
        self.mock(|when, then| {
            when.method(POST)
                .path_matches(Regex::new(r"/v1/trucks/.{36}/driveState").unwrap())
                .header("X-API-KEY", "API_KEY");
            then.status(201);
        })
    }
    fn list_driver_cards_mock(&self) -> Mock {
        self.mock(|when, then| {
            when.method(GET)
                .path("/v1/trucks/3ffaf18c-69e4-4f8a-9179-9aec5bc96e1c/driverCards")
                .header("X-API-KEY", "API_KEY");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body_obj(&[TruckDriverCard {
                    id: "1069619335000001".to_string().clone(),
                    timestamp: chrono::Utc::now().timestamp(),
                }]);
        })
    }
    fn delete_driver_card_mock(&self) -> Mock {
        self.mock(|when, then| {
            when.method(DELETE)
                .path(format!(
                    "/v1/trucks/3ffaf18c-69e4-4f8a-9179-9aec5bc96e1c/driverCards/{}",
                    "1069619335000001".to_string().clone()
                ))
                .header("X-API-KEY", "API_KEY");
            then.status(204);
        })
    }
}

#[cfg(test)]
pub fn get_temp_dir() -> tempfile::TempDir {
    return tempdir().unwrap();
}

#[cfg(test)]
pub fn get_temp_dir_path() -> std::path::PathBuf {
    return get_temp_dir().path().to_path_buf();
}

/// Wait until the condition is met or timeout of 5s is reached.
///
/// This function will keep calling the condition closure every 100ms until the condition returns true or the timeout of 5s is reached.
///
/// # Arguments
/// * `condition` - A closure that returns a tuple of a boolean and a value
///
/// # Returns
/// The value returned by the condition closure
#[cfg(test)]
pub fn wait_until<T>(condition: impl Fn() -> (bool, T)) -> T {
    use std::{thread, time::Duration};

    let interval = Duration::from_millis(100);
    let timeout = Duration::from_secs(5);
    let start = std::time::Instant::now();
    loop {
        let (result, data) = condition();
        if result {
            return data;
        }
        if start.elapsed() > timeout {
            panic!("Timeout");
        }
        thread::sleep(interval);
    }
}

/// Wait until the condition is met or given timeout is reached.
///
/// This function will keep calling the condition closure every 100ms until the condition returns true or the given timeout is reached.
///
/// # Arguments
/// * `condition` - A closure that returns a tuple of a boolean and a value
/// * `timeout` - Timeout in milliseconds
///
/// # Returns
/// The value returned by the condition closure
#[cfg(test)]
pub fn wait_until_timeout<T>(condition: impl Fn() -> (bool, T), timeout: u64) -> T {
    use std::{thread, time::Duration};

    let interval = Duration::from_millis(100);
    let timeout = Duration::from_millis(timeout);
    let start = std::time::Instant::now();
    loop {
        let (result, data) = condition();
        if result {
            return data;
        }
        if start.elapsed() > timeout {
            panic!("Timeout");
        }
        thread::sleep(interval);
    }
}
