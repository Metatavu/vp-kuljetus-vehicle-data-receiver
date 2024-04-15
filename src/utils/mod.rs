use vehicle_management_service::apis::configuration::Configuration;

pub mod avl_record_builder;
pub mod avl_frame_builder;
pub mod avl_packet;

#[cfg(test)]
pub mod imei;

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

/// Reads string environment variable
///
/// Panics if the environment variable is not set.
///
/// # Arguments
/// * `key` - Environment variable key
///
/// # Returns
/// * `String` - Environment variable value
pub fn read_string_env_variable(key: &str) -> String {
    match std::env::var(key) {
        Ok(value) => value,
        Err(_) => panic!("{} environment variable not set", key)
    }
}

/// Reads boolean environment variable
///
/// Panics if the environment variable is not set.
///
/// # Arguments
/// * `key` - Environment variable key
///
/// # Returns
/// * `bool` - Environment variable value
pub fn read_bool_env_variable(key: &str) -> bool {
    match std::env::var(key) {
        Ok(value) => value.parse().unwrap(),
        Err(_) => panic!("{} environment variable not set", key)
    }
}

/// Gets the API configuration for VP-Kuljetus Vehicle Management Service
///
/// # Returns
/// * [`Configuration`] - The API configuration
pub fn get_vehicle_management_api_config() -> Configuration {
    let api_key = vehicle_management_service::apis::configuration::ApiKey {
      prefix: None,
      key: read_string_env_variable("VEHICLE_MANAGEMENT_SERVICE_API_KEY")
    };
    Configuration {
      base_path: read_string_env_variable("API_BASE_URL"),
      api_key: Some(api_key),
      ..Default::default()
    }
}