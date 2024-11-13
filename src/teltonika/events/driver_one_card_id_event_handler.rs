use nom_teltonika::AVLEventIO;
use vehicle_management_service::{
    apis::{
        trucks_api::{
            create_truck_driver_card, delete_truck_driver_card, CreateTruckDriverCardError,
            CreateTruckDriverCardParams, DeleteTruckDriverCardError, DeleteTruckDriverCardParams,
        },
        Error,
    },
    models::TruckDriverCard,
};

use crate::{
    telematics_cache::Cacheable,
    teltonika::driver_card_events_to_truck_driver_card,
    utils::{api::get_truck_driver_card_id, VEHICLE_MANAGEMENT_API_CONFIG},
};

use super::teltonika_event_handlers::TeltonikaEventHandler;

pub struct DriverOneCardIdEventHandler;

impl DriverOneCardIdEventHandler {
    async fn create_truck_driver_card(
        &self,
        truck_id: String,
        truck_driver_card: TruckDriverCard,
    ) -> Result<(), DriverOneCardIdEventHandlerError> {
        let params = CreateTruckDriverCardParams {
            truck_id,
            truck_driver_card,
        };
        let res = create_truck_driver_card(&VEHICLE_MANAGEMENT_API_CONFIG, params).await;

        return match res {
            Ok(_) => Ok(()),
            Err(error) => match &error {
                // From our point of view 409 is a successful response, as it means that the driver card was successfully delivered to API.
                Error::ResponseError(err) => match err.status {
                    reqwest::StatusCode::CONFLICT => Ok(()),
                    _ => Err(error),
                },
                _ => Err(error),
            },
        }
        .map_err(|err| DriverOneCardIdEventHandlerError::CreateTruckDriverCardError(err));
    }

    async fn delete_truck_driver_card(
        &self,
        truck_id: String,
        x_removed_at: String,
    ) -> Result<(), DriverOneCardIdEventHandlerError> {
        let driver_card_id = get_truck_driver_card_id(truck_id.clone())
            .await
            .expect(&format!("Failed to get driver card id for truck {truck_id}"));
        let params = DeleteTruckDriverCardParams {
            truck_id,
            driver_card_id,
            x_removed_at,
        };
        let res = delete_truck_driver_card(&VEHICLE_MANAGEMENT_API_CONFIG, params).await;

        return match res {
            Ok(_) => Ok(()),
            Err(error) => match &error {
                Error::ResponseError(err) => match err.status {
                    reqwest::StatusCode::NOT_FOUND => Ok(()),
                    _ => Err(error),
                },
                _ => Err(error),
            },
        }
        .map_err(|err| DriverOneCardIdEventHandlerError::DeleteTruckDriverCardError(err));
    }
}

#[derive(Debug)]
pub enum DriverOneCardIdEventHandlerError {
    CreateTruckDriverCardError(Error<CreateTruckDriverCardError>),
    DeleteTruckDriverCardError(Error<DeleteTruckDriverCardError>),
}

impl TeltonikaEventHandler<TruckDriverCard, DriverOneCardIdEventHandlerError> for DriverOneCardIdEventHandler {
    fn get_event_ids(&self) -> Vec<u16> {
        vec![195, 196]
    }

    fn get_trigger_event_id(&self) -> Option<u16> {
        Some(187)
    }

    async fn send_event(
        &self,
        event_data: &TruckDriverCard,
        truck_id: String,
    ) -> Result<(), DriverOneCardIdEventHandlerError> {
        match &event_data.removed_at {
            Some(removed_at) => self.delete_truck_driver_card(truck_id, removed_at.clone()).await,
            None => self.create_truck_driver_card(truck_id, event_data.clone()).await,
        }
    }

    fn process_event_data(
        &self,
        trigger_event_id: u16,
        events: &Vec<&AVLEventIO>,
        timestamp: i64,
        _: &str,
    ) -> Option<TruckDriverCard> {
        match trigger_event_id {
            187 => driver_card_events_to_truck_driver_card(timestamp, events),
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
