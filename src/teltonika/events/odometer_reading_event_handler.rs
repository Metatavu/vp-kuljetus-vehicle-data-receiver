use nom_teltonika::AVLEventIO;
use vehicle_management_service::{
    apis::{
        trucks_api::{
            create_truck_odometer_reading, CreateTruckOdometerReadingError, CreateTruckOdometerReadingParams,
        },
        Error,
    },
    models::TruckOdometerReading,
};

use crate::{
    telematics_cache::Cacheable, teltonika::avl_event_io_value_to_u32, utils::get_vehicle_management_api_config,
};

use super::teltonika_event_handlers::TeltonikaEventHandler;

pub struct OdometerReadingEventHandler;

impl TeltonikaEventHandler<TruckOdometerReading, Error<CreateTruckOdometerReadingError>>
    for OdometerReadingEventHandler
{
    fn get_event_ids(&self) -> Vec<u16> {
        vec![192]
    }

    async fn send_event(
        &self,
        event_data: &TruckOdometerReading,
        truck_id: String,
        _: &str,
    ) -> Result<(), Error<CreateTruckOdometerReadingError>> {
        create_truck_odometer_reading(
            &get_vehicle_management_api_config(),
            CreateTruckOdometerReadingParams {
                truck_id,
                truck_odometer_reading: event_data.clone(),
            },
        )
        .await
    }

    fn process_event_data(
        &self,
        _trigger_event_id: u16,
        events: &Vec<&AVLEventIO>,
        timestamp: i64,
        _imei: &str,
    ) -> Option<TruckOdometerReading> {
        let event = events.first().expect("Received empty odometer reading event");
        Some(TruckOdometerReading::new(
            timestamp,
            avl_event_io_value_to_u32(&event.value) as i32,
        ))
    }
}

impl Cacheable for TruckOdometerReading {
    fn get_file_path() -> String
    where
        Self: Sized,
    {
        String::from("truck_odometer_reading_cache.json")
    }

    fn from_teltonika_record(_record: &nom_teltonika::AVLRecord) -> Option<Self> {
        todo!()
    }
}

impl Cacheable for Vec<TruckOdometerReading> {
    fn get_file_path() -> String
    where
        Self: Sized,
    {
        String::from("truck_speed_cache.json")
    }

    fn from_teltonika_record(_record: &nom_teltonika::AVLRecord) -> Option<Self> {
        todo!()
    }
}
