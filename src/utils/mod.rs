use std::str::FromStr;

use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use vehicle_management_service::apis::configuration::Configuration;

pub mod api;
pub mod avl_frame_builder;
pub mod avl_packet;
pub mod avl_record_builder;
#[cfg(test)]
pub mod imei;
#[cfg(test)]
pub mod test_utils;

lazy_static! {
    /// The default API configuration for VP-Kuljetus Vehicle Management Service
    pub static ref VEHICLE_MANAGEMENT_API_CONFIG: Configuration = get_vehicle_management_api_config();
}

/// Converts a hex string to a byte vector
///
/// # Arguments
/// * `input` - The hex string to convert
///
/// # Returns
/// * `Vec<u8>` - The byte vector
#[cfg(test)]
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

/// Reads environment variable and parses it to the desired type
///
/// Panics if the environment variable is not set
///
/// # Arguments
/// * `key` - The environment variable key
///
/// # Returns
/// * `T` - The parsed environment variable
pub fn read_env_variable<T: FromStr>(key: &str) -> T {
    match std::env::var(key) {
        Ok(value) => parse_env_variable(value),
        Err(_) => panic!("{} environment variable not set", key),
    }
}

/// Reads environment variable and parses it to the desired type wrapped in an Option
///
/// # Arguments
/// * `key` - The environment variable key
///
/// # Returns
/// * `Option<T>` - The parsed environment variable
pub fn read_optional_env_variable<T: FromStr>(key: &str) -> Option<T> {
    match std::env::var(key) {
        Ok(value) => Some(parse_env_variable(value)),
        Err(_) => None,
    }
}

/// Parses an environment variable to the desired type
///
/// Panics if the parsing fails
///
/// # Arguments
/// * `value` - The environment variable value
fn parse_env_variable<T: FromStr>(value: String) -> T {
    match value.parse() {
        Ok(parsed) => parsed,
        Err(_) => panic!("Failed to parse environment variable"),
    }
}

/// Reads environment variable and parses it to the desired type with a default value
///
/// If the environment variable is not set, the default value is returned.
///
/// # Arguments
/// * `key` - The environment variable key
/// * `default_value` - The default value
///
/// # Returns
/// * `T` - The parsed environment variable or the default value
pub fn read_env_variable_with_default_value<T: FromStr>(key: &str, default_value: T) -> T {
    match read_optional_env_variable(key) {
        Some(value) => value,
        None => default_value,
    }
}

/// Gets the API configuration for VP-Kuljetus Vehicle Management Service
///
/// # Returns
/// * [`Configuration`] - The API configuration
pub fn get_vehicle_management_api_config() -> Configuration {
    let api_key = vehicle_management_service::apis::configuration::ApiKey {
        prefix: None,
        key: read_env_variable("VEHICLE_MANAGEMENT_SERVICE_API_KEY"),
    };
    Configuration {
        base_path: read_env_variable("API_BASE_URL"),
        api_key: Some(api_key),
        ..Default::default()
    }
}

/// Converts a timestamp (seconds) to a DateTime<Utc>.
///
/// Panics if the timestamp is invalid.
///
/// # Arguments
/// * `timestamp` - The timestamp in seconds
///
/// # Returns
/// * `DateTime<Utc>` - The DateTime<Utc> representation of the timestamp
pub fn date_time_from_timestamp(timestamp: i64) -> DateTime<Utc> {
    return DateTime::from_timestamp(timestamp, 0).expect(&format!("Invalid timestamp {timestamp}"));
}

#[cfg(test)]
mod tests {
    use chrono::{LocalResult, TimeZone};

    #[test]
    fn test_date_time_from_timestamp() {
        let now = match chrono::Utc.with_ymd_and_hms(2024, 11, 13, 8, 5, 32) {
            LocalResult::None => panic!("Invalid date time"),
            LocalResult::Single(dt) => dt,
            LocalResult::Ambiguous(_, _) => panic!("Ambiguous date time"),
        };
        let timestamp = now.timestamp();
        let date_time = super::date_time_from_timestamp(timestamp);

        assert_eq!(date_time, now);
    }
}
