use log::{info, warn};
use nom_teltonika::AVLEventIO;
use vehicle_management_service::{
    apis::{
        trucks_api::{delete_truck_driver_card, DeleteTruckDriverCardError, DeleteTruckDriverCardParams},
        Error,
    },
    models::TruckDriverCard,
};

use crate::{
    telematics_cache::Cacheable,
    teltonika::{avl_event_io_value_to_u8, get_card_removal_time_from_event},
    utils::{api::get_truck_driver_card_id, VEHICLE_MANAGEMENT_API_CONFIG},
};

use super::teltonika_event_handlers::TeltonikaEventHandler;

pub struct DriverOneCardRemovedEventHandler;

impl TeltonikaEventHandler<TruckDriverCard, Error<DeleteTruckDriverCardError>> for DriverOneCardRemovedEventHandler {
    fn get_event_ids(&self) -> Vec<u16> {
        vec![187]
    }

    fn get_trigger_event_id(&self) -> Option<u16> {
        Some(187)
    }

    async fn send_event(
        &self,
        event_data: &TruckDriverCard,
        truck_id: String,
        imei: &str,
    ) -> Result<(), Error<DeleteTruckDriverCardError>> {
        let Some(removed_at) = event_data.removed_at.clone() else {
            return Ok(());
        };

        let params = DeleteTruckDriverCardParams {
            driver_card_id: get_truck_driver_card_id(truck_id.clone()).await.unwrap(),
            truck_id,
            x_removed_at: removed_at,
        };
        let result = delete_truck_driver_card(&VEHICLE_MANAGEMENT_API_CONFIG, params).await;

        match result {
            Ok(_) => {
                info!(target: imei, "Driver card removed!");
                Ok(())
            }
            Err(error) => match &error {
                // From our point of view 404 is a successful response, as it means that the driver card was successfully removed from API.
                Error::ResponseError(err) => match err.status {
                    reqwest::StatusCode::NOT_FOUND => Ok(()),
                    _ => Err(error),
                },
                _ => Err(error),
            },
        }
    }

    fn process_event_data(
        &self,
        trigger_event_id: u16,
        events: &Vec<&AVLEventIO>,
        timestamp: i64,
        imei: &str,
    ) -> Option<TruckDriverCard> {
        let Some(driver_one_card_presence_event) = events.iter().find(|event| event.id == 187) else {
            warn!(target: imei, "Driver one card presence event not found in events.");
            return None;
        };
        let driver_one_card_presence = avl_event_io_value_to_u8(&driver_one_card_presence_event.value);
        let removed_at = get_card_removal_time_from_event(driver_one_card_presence_event, timestamp);
        match (trigger_event_id, driver_one_card_presence) {
            (187, 0) => Some(TruckDriverCard {
                id: String::new(),
                timestamp,
                removed_at,
            }),
            _ => None,
        }
    }
}

impl Cacheable for TruckDriverCard {
    fn get_file_path() -> String
    where
        Self: Sized,
    {
        String::from("truck_driver_card_cache.json")
    }

    fn from_teltonika_record(_: &nom_teltonika::AVLRecord) -> Option<Self> {
        None
    }
}

impl Cacheable for Vec<TruckDriverCard> {
    fn get_file_path() -> String
    where
        Self: Sized,
    {
        String::from("truck_driver_card_cache.json")
    }

    fn from_teltonika_record(_: &nom_teltonika::AVLRecord) -> Option<Self> {
        None
    }
}
