use log::debug;
use nom_teltonika::AVLEventIO;
use vehicle_management_service::{
    apis::{
        trucks_api::{create_drive_state, CreateDriveStateError, CreateDriveStateParams},
        Error,
    },
    models::{Trackable, TrackableType, TruckDriveState, TruckDriveStateEnum},
};

use crate::{
    teltonika::{driver_card_events_to_truck_driver_card, FromAVLEventIoValue},
    utils::get_vehicle_management_api_config,
    Listener,
};

use super::teltonika_event_handlers::TeltonikaEventHandler;

#[derive(Debug)]
pub struct DriverOneDriveStateEventHandler;

impl TeltonikaEventHandler<TruckDriveState, Error<CreateDriveStateError>> for DriverOneDriveStateEventHandler {
    fn get_event_ids(&self, _listener: &Listener) -> Vec<u16> {
        vec![184, 195, 196]
    }

    fn get_event_handler_name(&self) -> String {
        return "driver_one_drive_state".to_string();
    }

    async fn send_event(
        &self,
        event_data: &TruckDriveState,
        trackable: Trackable,
        _: &str,
    ) -> Result<(), Error<CreateDriveStateError>> {
        if trackable.trackable_type == TrackableType::Towable {
            return Ok(());
        }
        create_drive_state(
            &get_vehicle_management_api_config(),
            CreateDriveStateParams {
                truck_id: trackable.id.to_string().clone(),
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
        _listener: &Listener,
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

    use crate::{
        teltonika::events::teltonika_event_handlers::TeltonikaEventHandler, utils::imei::get_random_imei, Listener,
    };

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

        let event_with_card_present =
            handler.process_event_data(0, &events, timestamp, &imei, &Listener::TeltonikaFMC650);
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

        let event_without_card_present =
            handler.process_event_data(0, &events, timestamp, &imei, &Listener::TeltonikaFMC650);

        // There is driver state event so the processed event should be Some
        assert!(event_without_card_present.is_some());

        events.remove(0);

        let event_without_card_present_event =
            handler.process_event_data(0, &events, timestamp, &imei, &Listener::TeltonikaFMC650);

        // There is driver state event so the processed event should be Some
        assert!(event_without_card_present_event.is_some());
    }
}
