use nom_teltonika::AVLEventIO;
use vehicle_management_service::{
    apis::{
        trucks_api::{create_truck_speed, CreateTruckSpeedError, CreateTruckSpeedParams},
        Error,
    },
    models::{Trackable, TrackableType, TruckSpeed},
};

use super::teltonika_event_handlers::TeltonikaEventHandler;
use crate::{teltonika::avl_event_io_value_to_u64, utils::get_vehicle_management_api_config, Listener};

#[derive(Debug)]
pub struct SpeedEventHandler;

impl TeltonikaEventHandler<TruckSpeed, Error<CreateTruckSpeedError>> for SpeedEventHandler {
    fn get_event_ids(&self, _listener: &Listener) -> Vec<u16> {
        vec![191]
    }

    fn get_event_handler_name(&self) -> String {
        return "speed".to_string();
    }

    async fn send_event(
        &self,
        event_data: &TruckSpeed,
        trackable: Trackable,
        _: &str,
    ) -> Result<(), Error<CreateTruckSpeedError>> {
        if trackable.trackable_type == TrackableType::Towable {
            return Ok(());
        }
        create_truck_speed(
            &get_vehicle_management_api_config(),
            CreateTruckSpeedParams {
                truck_id: trackable.id.to_string().clone(),
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
        _imei: &str,
        _listener: &Listener,
    ) -> Option<TruckSpeed> {
        let event = events.first().expect("Received empty speed event");
        Some(TruckSpeed::new(
            timestamp,
            avl_event_io_value_to_u64(&event.value) as f32,
        ))
    }
}
