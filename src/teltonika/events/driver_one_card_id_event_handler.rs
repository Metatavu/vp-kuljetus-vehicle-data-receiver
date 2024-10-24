use nom_teltonika::AVLEventIO;
use vehicle_management_service::{
    apis::{
        trucks_api::{CreateTruckDriverCardError, CreateTruckDriverCardParams},
        Error,
    },
    models::TruckDriverCard,
};

use crate::{
    telematics_cache::Cacheable, teltonika::driver_card_events_to_truck_driver_card,
    utils::get_vehicle_management_api_config,
};

use super::teltonika_event_handlers::TeltonikaEventHandler;

pub struct DriverOneCardIdEventHandler;

impl TeltonikaEventHandler<TruckDriverCard, Error<CreateTruckDriverCardError>>
    for DriverOneCardIdEventHandler
{
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
    ) -> Result<(), Error<CreateTruckDriverCardError>> {
        let res = vehicle_management_service::apis::trucks_api::create_truck_driver_card(
            &get_vehicle_management_api_config(),
            CreateTruckDriverCardParams {
                truck_id: truck_id.clone(),
                truck_driver_card: event_data.clone(),
            },
        )
        .await;
        match res {
            Ok(_) => Ok(()),
            Err(error) => match &error {
                // API returns a 409 if the truck already has a driver card. At least for now, swallow them silently and continue.
                Error::ResponseError(err) => {
                    if err.status.as_u16() == reqwest::StatusCode::CONFLICT {
                        return Ok(());
                    } else {
                        return Err(error);
                    }
                }
                _ => Err(error),
            },
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
