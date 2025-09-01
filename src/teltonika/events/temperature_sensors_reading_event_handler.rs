use log::{debug, info, warn};
use nom_teltonika::AVLEventIO;
use vehicle_management_service::{
    apis::{
        temperature_readings_api::{
            create_temperature_reading, CreateTemperatureReadingError, CreateTemperatureReadingParams,
        },
        Error,
    },
    models::{TemperatureReading, TemperatureReadingSourceType, Trackable, TrackableType},
};

use crate::{
    teltonika::{avl_event_io_value_to_u16, avl_event_io_value_to_u64},
    utils::get_vehicle_management_api_config,
    Listener,
};

use super::{teltonika_event_handlers::TeltonikaEventHandler, TeltonikaTemperatureSensors};

#[derive(Debug)]
pub struct TemperatureSensorsReadingEventHandler;

impl TemperatureSensorsReadingEventHandler {
    fn parse_readings_from_events(
        &self,
        log_target: &str,
        events: &Vec<&AVLEventIO>,
        sensor: &TeltonikaTemperatureSensors,
        timestamp: i64,
        listener: &Listener,
    ) -> Option<TemperatureReading> {
        let imei = log_target
            .split("-")
            .collect::<Vec<&str>>()
            .first()
            .unwrap()
            .trim()
            .to_string();

        let hardware_sensor_id = match events
            .iter()
            .find(|event| event.id == sensor.hardware_sensor_io_event_id(listener))
        {
            Some(hardware_sensor_id) => Some(avl_event_io_value_to_u64(&hardware_sensor_id.value)),
            None => {
                debug!(target: log_target, "No hardware sensor ID found for {sensor:#?}");
                return None;
            }
        };
        let temperature = match events
            .iter()
            .find(|event| event.id == sensor.temperature_reading_io_event_id())
        {
            Some(temperature) => Some(avl_event_io_value_to_u16(&temperature.value)),
            None => {
                debug!(target: log_target, "No temperature found for {sensor:#?}");
                return None;
            }
        };

        if hardware_sensor_id.is_none() || temperature.is_none() {
            return None;
        }
        let hardware_sensor_id = hardware_sensor_id.unwrap();
        let temperature = temperature.unwrap();
        if hardware_sensor_id == 0 {
            return None;
        }

        return Some(TemperatureReading::new(
            imei,
            hardware_sensor_id.to_string(),
            temperature as f32 * 0.1,
            timestamp,
            TemperatureReadingSourceType::Truck,
        ));
    }
}

impl TeltonikaEventHandler<Vec<TemperatureReading>, Error<CreateTemperatureReadingError>>
    for TemperatureSensorsReadingEventHandler
{
    fn require_all_events(&self) -> bool {
        false
    }

    fn get_event_handler_name(&self) -> String {
        return "temperature_sensors_reading".to_string();
    }

    fn get_event_ids(&self, listener: &Listener) -> Vec<u16> {
        match listener {
            Listener::TeltonikaFMC650 => vec![
                62, // Temperature sensor 1 ID
                72, // Temperature sensor 1 reading
                63, // Temperature sensor 2 ID
                73, // Temperature sensor 2 reading
                64, // Temperature sensor 3 ID
                74, // Temperature sensor 3 reading
                65, // Temperature sensor 4 ID
                75, // Temperature sensor 4 reading
                5,  // Temperature sensor 5 ID
                6,  // Temperature sensor 5 reading
                7,  // Temperature sensor 6 ID
                8,  // Temperature sensor 6 reading
            ],
            Listener::TeltonikaFMC234 => vec![
                76, // Temperature sensor 1 ID
                72, // Temperature sensor 1 reading
                77, // Temperature sensor 2 ID
                73, // Temperature sensor 2 reading
                79, // Temperature sensor 3 ID
                74, // Temperature sensor 3 reading
                71, // Temperature sensor 4 ID
                75, // Temperature sensor 4 reading
            ],
        }
    }

    async fn send_event(
        &self,
        event_data: &Vec<TemperatureReading>,
        trackable: Trackable,
        log_target: &str,
    ) -> Result<(), Error<CreateTemperatureReadingError>> {
        let mut errors = Vec::new();
        debug!(target: log_target, "Amount of readings: {}", event_data.len());

        for reading in event_data {
            let mut reading = reading.clone();
            reading.source_type = match trackable.trackable_type {
                TrackableType::Towable => TemperatureReadingSourceType::Towable,
                TrackableType::Truck => TemperatureReadingSourceType::Truck,
            };
            debug!(target: log_target, "Got vehicle management API config for temperature sending");
            let config = &get_vehicle_management_api_config();
            debug!(target: log_target, "Sending reading to server");
            match create_temperature_reading(
                config,
                CreateTemperatureReadingParams {
                    temperature_reading: reading.clone(),
                },
            )
            .await
            {
                Ok(_) => {
                    debug!(target: log_target, "Successfully sent temperature reading");
                }
                Err(e) => {
                    debug!(target: log_target, "Failed to send temperature reading");
                    errors.push(e);
                    break;
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
        events: &Vec<&AVLEventIO>,
        timestamp: i64,
        log_target: &str,
        listener: &Listener,
    ) -> Option<Vec<TemperatureReading>> {
        let mut readings = Vec::new();
        for sensor in TeltonikaTemperatureSensors::iterator() {
            readings.push(self.parse_readings_from_events(log_target, events, sensor, timestamp, listener));
        }
        debug!(target: log_target, "Parsed readings: {:?}", readings);
        Some(
            readings
                .iter()
                .filter(|x| x.is_some())
                .map(|x| x.clone().unwrap())
                .collect(),
        )
    }
}
