use log::{info, warn};
use nom_teltonika::AVLEventIO;
use uuid::Uuid;
use vehicle_management_service::{
    apis::{
        trucks_api::{
            create_truck_driver_card, delete_truck_driver_card, list_truck_driver_cards, CreateTruckDriverCardError,
            CreateTruckDriverCardParams, DeleteTruckDriverCardError, DeleteTruckDriverCardParams,
            ListTruckDriverCardsError, ListTruckDriverCardsParams,
        },
        Error,
    },
    models::{Trackable, TrackableType, TruckDriverCard},
};

use crate::{
    teltonika::{avl_event_io_value_to_u8, driver_card_events_to_truck_driver_card},
    utils::{
        api::fetch_all_driver_cards_in_truck, date_time_from_timestamp, get_vehicle_management_api_config,
        VEHICLE_MANAGEMENT_API_CONFIG,
    },
    Listener,
};

use super::teltonika_event_handlers::TeltonikaEventHandler;

#[derive(Debug)]
pub struct DriverOneCardEventHandler;

impl DriverOneCardEventHandler {
    fn process_card_removed_event_data(
        &self,
        events: &Vec<&AVLEventIO>,
        timestamp: i64,
        log_target: &str,
    ) -> Option<TruckDriverCard> {
        let Some(driver_one_card_presence_event) = events.iter().find(|event| event.id == 187) else {
            warn!(target: log_target, "Couldn't process card removed event; Driver one card presence event not found in events: {:?}", events);
            return None;
        };
        let driver_one_card_presence = avl_event_io_value_to_u8(&driver_one_card_presence_event.value);
        let mut truck_driver_card = TruckDriverCard::new(String::new(), timestamp);
        truck_driver_card.removed_at = Some(date_time_from_timestamp(timestamp).to_rfc3339());

        return match driver_one_card_presence {
            0 => Some(truck_driver_card),
            _ => None,
        };
    }

    async fn create_truck_driver_card(
        &self,
        truck_id: Uuid,
        truck_driver_card: TruckDriverCard,
        imei: &str,
    ) -> Result<(), DriverOneCardIdEventHandlerError> {
        let params = CreateTruckDriverCardParams {
            truck_id: truck_id.to_string(),
            truck_driver_card,
        };
        let res = create_truck_driver_card(&VEHICLE_MANAGEMENT_API_CONFIG, params).await;

        return match res {
            Ok(_) => {
                info!(target: imei, "Driver card inserted successfully!");
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
        }
        .map_err(|err| DriverOneCardIdEventHandlerError::CreateTruckDriverCardError(err));
    }

    async fn delete_truck_driver_card(
        &self,
        truck_id: Uuid,
        x_removed_at: String,
        log_target: &str,
    ) -> Result<(), DriverOneCardIdEventHandlerError> {
        let truck_id = truck_id.clone().to_string();

        let driver_cards_result = list_truck_driver_cards(
            &get_vehicle_management_api_config(),
            ListTruckDriverCardsParams {
                truck_id: truck_id.to_string(),
            },
        )
        .await;

        match driver_cards_result {
            Ok(driver_cards) => {
                let driver_cards = driver_cards
                    .iter()
                    .filter(|card| card.removed_at.is_none())
                    .collect::<Vec<_>>();

                let driver_card = driver_cards.first();
                let driver_card_id = match driver_card {
                    Some(card) => card.id.clone(),
                    None => {
                        info!(target: log_target, "No active driver card found for truck [{}], nothing to remove", truck_id);
                        return Ok(());
                    }
                };
                let params = DeleteTruckDriverCardParams {
                    truck_id,
                    driver_card_id,
                    x_removed_at,
                };
                let res = delete_truck_driver_card(&VEHICLE_MANAGEMENT_API_CONFIG, params).await;

                return match res {
                    Ok(_) => {
                        info!(target: log_target, "Driver card removed successfully!");
                        Ok(())
                    }
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
            Err(error) => {
                warn!(target: log_target, "Failed to get driver cards for truck [{}]: {}", truck_id, error);
                return Err(DriverOneCardIdEventHandlerError::ListTruckDriverCardsError(error));
            }
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum DriverOneCardIdEventHandlerError {
    CreateTruckDriverCardError(Error<CreateTruckDriverCardError>),
    DeleteTruckDriverCardError(Error<DeleteTruckDriverCardError>),
    ListTruckDriverCardsError(Error<ListTruckDriverCardsError>),
}

impl TeltonikaEventHandler<TruckDriverCard, DriverOneCardIdEventHandlerError> for DriverOneCardEventHandler {
    fn get_event_ids(&self, _listener: &Listener) -> Vec<u16> {
        vec![195, 196, 187]
    }

    fn get_trigger_event_ids(&self) -> Vec<u16> {
        vec![187, 195]
    }

    fn get_event_handler_name(&self) -> String {
        return "driver_one_card".to_string();
    }

    async fn send_event(
        &self,
        event_data: &TruckDriverCard,
        trackable: Trackable,
        log_target: &str,
    ) -> Result<(), DriverOneCardIdEventHandlerError> {
        if trackable.trackable_type == TrackableType::Towable {
            return Ok(());
        }
        match &event_data.removed_at {
            Some(removed_at) => {
                self.delete_truck_driver_card(trackable.id, removed_at.clone(), log_target)
                    .await
            }
            None => {
                self.create_truck_driver_card(trackable.id, event_data.clone(), log_target)
                    .await
            }
        }
    }

    fn process_event_data(
        &self,
        trigger_event_id: u16,
        events: &Vec<&AVLEventIO>,
        timestamp: i64,
        imei: &str,
        _listener: &Listener,
    ) -> Option<TruckDriverCard> {
        return match trigger_event_id {
            187 => self.process_card_removed_event_data(events, timestamp, imei),
            195 => driver_card_events_to_truck_driver_card(timestamp, events, imei),
            _ => None,
        };
    }
}
