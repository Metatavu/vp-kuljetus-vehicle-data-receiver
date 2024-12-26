use log::warn;
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

pub struct TemperatureSensor1ReadingEventHandler;

impl TeltonikaEventHandler<TemperatureReading, Error<CreateTemperatureReadingError>>
    for TemperatureSensor1ReadingEventHandler
{
    fn get_event_ids(&self) -> Vec<u16> {
        vec![62, 72]
    }

    async fn send_event(
        &self,
        event_data: &TemperatureReading,
        truck_id: String,
        imei: &str,
    ) -> Result<(), Error<CreateTemperatureReadingError>> {
        create_temperature_reading(
            &get_vehicle_management_api_config(),
            CreateTemperatureReadingParams {
                temperature_reading: event_data.clone(),
            },
        )
        .await
    }

    fn process_event_data(
        &self,
        trigger_event_id: u16,
        events: &Vec<&nom_teltonika::AVLEventIO>,
        timestamp: i64,
        imei: &str,
    ) -> Option<TemperatureReading> {
        let Some(mac_address) = events.iter().find(|event| event.id == 62) else {
            warn!(target: imei, "No MAC address found for temperature sensor 1 reading event");
        };
        let Some(temperature) = events.iter().find(|event| event.id == 72) else {
            warn!(target: imei, "No temperature found for temperature sensor 1 reading event");
        };
        let mac_address = avl_event_io_value_to_u64(&mac_address.value);
        let temperature = avl_event_io_value_to_u16(&temperature.value);
        Some(TemperatureReading::new(&mei, mac_address, temperature, timestamp))
    }
}

impl Cacheable for TemperatureReading {
    fn get_file_path() -> String
    where
        Self: Sized,
    {
        String::from("temperature_sensor_1_reading_cache.json")
    }
}

impl Cacheable for Vec<TemperatureReading> {
    fn get_file_path() -> String
    where
        Self: Sized,
    {
        String::from("temperature_sensor_1_reading_cache.json")
    }
}
