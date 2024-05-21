use nom_teltonika::AVLEventIO;
use vehicle_management_service::{
    apis::{
        trucks_api::{CreateTruckSpeedError, CreateTruckSpeedParams},
        Error,
    },
    models::TruckSpeed,
};

use super::{avl_event_io_value_to_u64, teltonika_event_handlers::TeltonikaEventHandler};
use crate::{telematics_cache::Cacheable, utils::get_vehicle_management_api_config};

pub struct SpeedEventHandler;

impl TeltonikaEventHandler<TruckSpeed, Error<CreateTruckSpeedError>> for SpeedEventHandler {
    fn get_event_ids(&self) -> Vec<u16> {
        vec![191]
    }

    async fn send_event(
        &self,
        event_data: &TruckSpeed,
        truck_id: String,
    ) -> Result<(), Error<CreateTruckSpeedError>> {
        vehicle_management_service::apis::trucks_api::create_truck_speed(
            &get_vehicle_management_api_config(),
            CreateTruckSpeedParams {
                truck_id,
                truck_speed: event_data.clone(),
            },
        )
        .await
    }

    fn process_event_data(
        &self,
        _trigger_event_id: u16,
        events: &Vec<&AVLEventIO>,
        timestamp: i64,
    ) -> Option<TruckSpeed> {
        let event = events.first().expect("Received empty speed event");
        Some(TruckSpeed {
            id: None,
            speed: avl_event_io_value_to_u64(&event.value) as f32,
            timestamp,
        })
    }
}

impl Cacheable for TruckSpeed {
    const FILE_PATH: &'static str = "truck_speed_cache.json";

    fn from_teltonika_record(_record: &nom_teltonika::AVLRecord) -> Option<Self> {
        None
    }
}
