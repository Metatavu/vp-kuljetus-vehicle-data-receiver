use log::{info, warn};
use nom_teltonika::AVLEventIO;
use vehicle_management_service::{
    apis::{
        trucks_api::{create_truck_driver_card, CreateTruckDriverCardError, CreateTruckDriverCardParams},
        Error,
    },
    models::TruckDriverCard,
};

use crate::{
    teltonika::{avl_event_io_value_to_u8, driver_card_events_to_truck_driver_card},
    utils::VEHICLE_MANAGEMENT_API_CONFIG,
};

use super::teltonika_event_handlers::TeltonikaEventHandler;

pub struct DriverOneCardInsertedEventHandler;

impl TeltonikaEventHandler<TruckDriverCard, Error<CreateTruckDriverCardError>> for DriverOneCardInsertedEventHandler {
    fn get_event_ids(&self) -> Vec<u16> {
        vec![195, 196, 187]
    }

    fn get_trigger_event_id(&self) -> Option<u16> {
        Some(195)
    }

    async fn send_event(
        &self,
        event_data: &TruckDriverCard,
        truck_id: String,
        imei: &str,
    ) -> Result<(), Error<CreateTruckDriverCardError>> {
        let params = CreateTruckDriverCardParams {
            truck_id,
            truck_driver_card: event_data.clone(),
        };
        let result = create_truck_driver_card(&VEHICLE_MANAGEMENT_API_CONFIG, params).await;

        return match result {
            Ok(_) => {
                info!(target: imei, "Driver card inserted!");
                Ok(())
            }
            Err(error) => match &error {
                // From our point of view 409 is a successful response, as it means that the driver card was successfully delivered to API.
                Error::ResponseError(err) => match err.status {
                    reqwest::StatusCode::CONFLICT => Ok(()),
                    _ => Err(error),
                },
                _ => Err(error),
            },
        };
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
        match (trigger_event_id, driver_one_card_presence) {
            (195, 1) => driver_card_events_to_truck_driver_card(timestamp, events),
            _ => None,
        }
    }
}
