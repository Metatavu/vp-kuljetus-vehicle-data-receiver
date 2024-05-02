use nom_teltonika::AVLEventIO;
use vehicle_management_service::{
    apis::{
        trucks_api::{CreateTruckDriverCardError, CreateTruckDriverCardParams},
        Error,
    },
    models::TruckDriverCard,
};

use crate::{telematics_cache::Cacheable, utils::get_vehicle_management_api_config};

use super::{avl_event_io_value_to_u64, teltonika_event_handlers::TeltonikaEventHandler};

pub struct DriverOneCardIdEventHandler {}

impl TeltonikaEventHandler<TruckDriverCard, Error<CreateTruckDriverCardError>>
    for DriverOneCardIdEventHandler
{
    fn get_event_ids(&self) -> Vec<u16> {
        vec![195, 196]
    }

    async fn send_event(
        &self,
        event_data: TruckDriverCard,
        truck_id: String,
    ) -> Result<(), Error<CreateTruckDriverCardError>> {
        let res = vehicle_management_service::apis::trucks_api::create_truck_driver_card(
            &get_vehicle_management_api_config(),
            CreateTruckDriverCardParams {
                truck_id: truck_id.clone(),
                truck_driver_card: event_data,
            },
        )
        .await;
        match res {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn process_event_data(&self, events: &Vec<&AVLEventIO>, _: i64) -> TruckDriverCard {
        let driver_one_card_msb = events
            .iter()
            .find(|event| event.id == 195)
            .expect("Driver one card MSB event not found");
        let driver_one_card_lsb = events
            .iter()
            .find(|event| event.id == 196)
            .expect("Driver one card LSB event not found");
        let driver_one_card_msb =
            avl_event_io_value_to_u64(&driver_one_card_msb.value).to_be_bytes();
        let driver_one_card_lsb =
            avl_event_io_value_to_u64(&driver_one_card_lsb.value).to_be_bytes();
        let driver_one_card_part_1 = driver_one_card_msb
            .iter()
            .rev()
            .map(|byte| *byte as char)
            .collect::<String>();
        let driver_one_card_part_2 = driver_one_card_lsb
            .iter()
            .rev()
            .map(|byte| *byte as char)
            .collect::<String>();

        return TruckDriverCard {
            id: driver_one_card_part_2 + &driver_one_card_part_1,
        };
    }
}

impl Cacheable for TruckDriverCard {
    const FILE_PATH: &'static str = "truck_driver_card_cache.json";

    fn from_teltonika_events(events: Vec<&AVLEventIO>, _: i64) -> Option<Self> {
        let driver_one_card_msb = events
            .iter()
            .find(|event| event.id == 195)
            .expect("Driver one card MSB event not found");
        let driver_one_card_lsb = events
            .iter()
            .find(|event| event.id == 196)
            .expect("Driver one card LSB event not found");
        let driver_one_card_msb = avl_event_io_value_to_u64(&driver_one_card_msb.value);
        let driver_one_card_lsb = avl_event_io_value_to_u64(&driver_one_card_lsb.value);
        // Documentation says that the these are 8 bytes long (u64) that should be converted to ASCII. Let's see if it works with UTF-8
        let driver_one_card_part_2 = String::from_utf8(driver_one_card_msb.to_be_bytes().to_vec())
            .unwrap()
            .to_ascii_uppercase();
        let driver_one_card_part_1 = String::from_utf8(driver_one_card_lsb.to_be_bytes().to_vec())
            .unwrap()
            .to_ascii_uppercase();

        return Some(TruckDriverCard {
            id: driver_one_card_part_1 + &driver_one_card_part_2,
        });
    }

    fn from_teltonika_record(_: &nom_teltonika::AVLRecord) -> Option<Self> {
        None
    }
}
