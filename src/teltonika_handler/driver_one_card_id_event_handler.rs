use nom_teltonika::AVLEventIO;
use vehicle_management_service::{
    apis::{
        trucks_api::{CreateTruckDriverCardError, CreateTruckDriverCardParams},
        Error,
    },
    models::TruckDriverCard,
};

use crate::{telematics_cache::Cacheable, utils::get_vehicle_management_api_config};

use super::{
    driver_card_events_to_truck_driver_card, teltonika_event_handlers::TeltonikaEventHandler,
};

pub struct DriverOneCardIdEventHandler;

impl TeltonikaEventHandler<TruckDriverCard, Error<CreateTruckDriverCardError>>
    for DriverOneCardIdEventHandler
{
    fn get_event_ids(&self) -> Vec<u16> {
        vec![195, 196]
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
            Err(e) => Err(e),
        }
    }

    fn process_event_data(&self, events: &Vec<&AVLEventIO>, _: i64) -> TruckDriverCard {
        return driver_card_events_to_truck_driver_card(events);
    }
}

impl Cacheable for TruckDriverCard {
    const FILE_PATH: &'static str = "truck_driver_card_cache.json";

    fn from_teltonika_record(_: &nom_teltonika::AVLRecord) -> Option<Self> {
        None
    }
}
