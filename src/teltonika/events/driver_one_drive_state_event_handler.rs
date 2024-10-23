use log::debug;
use nom_teltonika::{AVLEventIO, AVLRecord};
use vehicle_management_service::{
    apis::{
        trucks_api::{CreateDriveStateError, CreateDriveStateParams},
        Error,
    },
    models::{TruckDriveState, TruckDriveStateEnum},
};

use crate::{
    telematics_cache::Cacheable,
    teltonika::{driver_card_events_to_truck_driver_card, FromAVLEventIoValue},
    utils::get_vehicle_management_api_config,
};

use super::teltonika_event_handlers::TeltonikaEventHandler;

pub struct DriverOneDriveStateEventHandler;

impl TeltonikaEventHandler<TruckDriveState, Error<CreateDriveStateError>>
    for DriverOneDriveStateEventHandler
{
    fn get_event_ids(&self) -> Vec<u16> {
        vec![184, 195, 196]
    }

    async fn send_event(
        &self,
        event_data: &TruckDriveState,
        truck_id: String,
    ) -> Result<(), Error<CreateDriveStateError>> {
        vehicle_management_service::apis::trucks_api::create_drive_state(
            &get_vehicle_management_api_config(),
            CreateDriveStateParams {
                truck_id: truck_id.clone(),
                truck_drive_state: event_data.clone(),
            },
        )
        .await
    }

    fn process_event_data(
        &self,
        _trigger_event_id: u16,
        events: &Vec<&AVLEventIO>,
        timestamp: i64,
        imei: &str,
    ) -> Option<TruckDriveState> {
        let Some(driver_card) = driver_card_events_to_truck_driver_card(timestamp, events) else {
            debug!(target: imei, "Driver card MSB or LSB was 0");

            return None;
        };
        let state_event = events
            .iter()
            .find(|event| event.id == 184)
            .expect("Driver one drive state event not found");
        let state = TruckDriveStateEnum::from_avl_event_io_value(&state_event.value);
        Some(TruckDriveState {
            id: None,
            timestamp,
            state,
            driver_id: None,
            driver_card_id: Some(driver_card.id),
        })
    }
}

impl Cacheable for TruckDriveState {
    fn get_file_path() -> String
    where
        Self: Sized,
    {
        String::from("truck_drive_state_cache.json")
    }

    fn from_teltonika_record(_: &AVLRecord) -> Option<Self> {
        None
    }
}
