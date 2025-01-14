use log::debug;
use nom_teltonika::AVLEventIO;
use vehicle_management_service::{
    apis::{
        trucks_api::{create_drive_state, CreateDriveStateError, CreateDriveStateParams},
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

impl TeltonikaEventHandler<TruckDriveState, Error<CreateDriveStateError>> for DriverOneDriveStateEventHandler {
    fn get_event_ids(&self) -> Vec<u16> {
        vec![184, 195, 196]
    }

    async fn send_event(
        &self,
        event_data: &TruckDriveState,
        truck_id: String,
        _: &str,
    ) -> Result<(), Error<CreateDriveStateError>> {
        create_drive_state(
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
        let Some(driver_card) = driver_card_events_to_truck_driver_card(timestamp, events, imei) else {
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

#[cfg(test)]
mod tests {
    use nom_teltonika::AVLEventIO;

    use crate::{teltonika::events::teltonika_event_handlers::TeltonikaEventHandler, utils::imei::get_random_imei};

    use super::DriverOneDriveStateEventHandler;

    #[test]
    fn test_process_event_data_with_card_present() {
        let handler = DriverOneDriveStateEventHandler;
        let timestamp = 1731485132;
        let imei = get_random_imei();
        let mut events = Vec::new();
        events.push(&AVLEventIO {
            id: 187,
            value: nom_teltonika::AVLEventIOValue::U8(1),
        });
        events.push(&AVLEventIO {
            id: 184,
            value: nom_teltonika::AVLEventIOValue::U8(0),
        });
        events.push(&AVLEventIO {
            id: 195,
            value: nom_teltonika::AVLEventIOValue::U64(3544392526090811699),
        });
        events.push(&AVLEventIO {
            id: 196,
            value: nom_teltonika::AVLEventIOValue::U64(3689908453225017393),
        });

        let event_with_card_present = handler.process_event_data(0, &events, timestamp, &imei);
        // There is driver state event so the processed event should be Some
        assert!(event_with_card_present.is_some());
    }

    #[test]
    fn test_process_event_data_without_card_presence() {
        let handler = DriverOneDriveStateEventHandler;
        let timestamp = 1731485132;
        let imei = get_random_imei();
        let mut events = Vec::new();
        events.push(&AVLEventIO {
            id: 187,
            value: nom_teltonika::AVLEventIOValue::U8(0),
        });
        events.push(&AVLEventIO {
            id: 184,
            value: nom_teltonika::AVLEventIOValue::U8(0),
        });
        events.push(&AVLEventIO {
            id: 195,
            value: nom_teltonika::AVLEventIOValue::U64(3544392526090811699),
        });
        events.push(&AVLEventIO {
            id: 196,
            value: nom_teltonika::AVLEventIOValue::U64(3689908453225017393),
        });

        let event_without_card_present = handler.process_event_data(0, &events, timestamp, &imei);

        // There is driver state event so the processed event should be Some
        assert!(event_without_card_present.is_some());

        events.remove(0);

        let event_without_card_present_event = handler.process_event_data(0, &events, timestamp, &imei);

        // There is driver state event so the processed event should be Some
        assert!(event_without_card_present_event.is_some());
    }
}

impl Cacheable for TruckDriveState {
    fn get_file_path() -> String
    where
        Self: Sized,
    {
        String::from("truck_drive_state_cache.json")
    }
}

impl Cacheable for Vec<TruckDriveState> {
    fn get_file_path() -> String
    where
        Self: Sized,
    {
        String::from("truck_drive_state_cache.json")
    }
}
