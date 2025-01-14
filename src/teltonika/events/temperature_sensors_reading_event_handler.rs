use log::warn;
use nom_teltonika::AVLEventIO;
use vehicle_management_service::{
    apis::{
        temperature_readings_api::{
            create_temperature_reading, CreateTemperatureReadingError, CreateTemperatureReadingParams,
        },
        Error,
    },
    models::TemperatureReading,
};

use crate::{
    telematics_cache::Cacheable,
    teltonika::{avl_event_io_value_to_u16, avl_event_io_value_to_u64},
    utils::get_vehicle_management_api_config,
};

use super::teltonika_event_handlers::TeltonikaEventHandler;

pub struct TemperatureSensorsReadingEventHandler;

impl TemperatureSensorsReadingEventHandler {
    fn parse_readings_from_events(
        &self,
        log_target: &str,
        events: &Vec<&AVLEventIO>,
        sensor_number: usize,
        timestamp: i64,
    ) -> Option<TemperatureReading> {
        let imei = log_target
            .split("-")
            .collect::<Vec<&str>>()
            .first()
            .unwrap()
            .trim()
            .to_string();
        let mac_address_event_id = 62 + sensor_number as u16;
        let temperature_event_id = 72 + sensor_number as u16;
        let Some(mac_address) = events.iter().find(|event| event.id == mac_address_event_id) else {
            warn!(target: log_target, "No MAC address found for temperature sensor 1 reading event");
            return None;
        };
        let Some(temperature) = events.iter().find(|event| event.id == temperature_event_id) else {
            warn!(target: log_target, "No temperature found for temperature sensor 1 reading event");
            return None;
        };
        let mac_address = avl_event_io_value_to_u64(&mac_address.value);
        let temperature = avl_event_io_value_to_u16(&temperature.value);

        return Some(TemperatureReading::new(
            imei,
            mac_address.to_string(),
            temperature as f32 * 0.1,
            timestamp,
        ));
    }
}

impl TeltonikaEventHandler<Vec<TemperatureReading>, Error<CreateTemperatureReadingError>>
    for TemperatureSensorsReadingEventHandler
{
    fn require_all_events(&self) -> bool {
        false
    }
    fn get_event_ids(&self) -> Vec<u16> {
        vec![
            62, // Temperature sensor 1 ID
            72, // Temperature sensor 1 reading
            63, // Temperature sensor 2 ID
            73, // Temperature sensor 2 reading
            64, // Temperature sensor 3 ID
            74, // Temperature sensor 3 reading
            65, // Temperature sensor 4 ID
            75, // Temperature sensor 4 reading
            66, // Temperature sensor 5 ID
            76, // Temperature sensor 5 reading
            67, // Temperature sensor 6 ID
            77, // Temperature sensor 6 reading
        ]
    }

    async fn send_event(
        &self,
        event_data: &Vec<TemperatureReading>,
        _: String,
        _: &str,
    ) -> Result<(), Error<CreateTemperatureReadingError>> {
        let mut errors = Vec::new();

        for reading in event_data {
            match create_temperature_reading(
                &get_vehicle_management_api_config(),
                CreateTemperatureReadingParams {
                    temperature_reading: reading.clone(),
                },
            )
            .await
            {
                Ok(_) => (),
                Err(e) => {
                    errors.push(e);
                }
            }
        }

        if !errors.is_empty() {
            let mapped_error = errors
                .iter()
                .map(|err| err.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            return Err(Error::Io(std::io::Error::new(std::io::ErrorKind::Other, mapped_error)));
        }
        Ok(())
    }

    fn process_event_data(
        &self,
        _: u16,
        events: &Vec<&nom_teltonika::AVLEventIO>,
        timestamp: i64,
        log_target: &str,
    ) -> Option<Vec<TemperatureReading>> {
        let mut readings = Vec::new();
        for number in 0..6 {
            readings.push(self.parse_readings_from_events(log_target, events, number, timestamp));
        }

        Some(
            readings
                .iter()
                .filter(|x| x.is_some())
                .map(|x| x.clone().unwrap())
                .collect(),
        )
    }
}

impl Cacheable for TemperatureReading {
    fn get_file_path() -> String
    where
        Self: Sized,
    {
        String::from("temperature_sensors_reading_cache.json")
    }
}

impl Cacheable for Vec<TemperatureReading> {
    fn get_file_path() -> String
    where
        Self: Sized,
    {
        String::from("temperature_sensors_reading_cache.json")
    }
}

impl Cacheable for Vec<Vec<TemperatureReading>> {
    fn get_file_path() -> String
    where
        Self: Sized,
    {
        String::from("temperature_sensors_reading_cache.json")
    }
}
