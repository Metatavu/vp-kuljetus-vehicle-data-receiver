use crate::{
    failed_events::{FailedEvent, FailedEventsHandler},
    teltonika::events::{
        DriverOneCardEventHandler, DriverOneDriveStateEventHandler, OdometerReadingEventHandler, SpeedEventHandler,
        TemperatureSensorsReadingEventHandler,
    },
    Listener,
};
use log::{debug, error};
use nom_teltonika::AVLEventIO;
use serde::{Deserialize, Serialize};
use sqlx::{MySql, Pool};
use std::fmt::Debug;
use vehicle_management_service::{
    apis::{
        temperature_readings_api::CreateTemperatureReadingError,
        trucks_api::{CreateDriveStateError, CreateTruckOdometerReadingError},
    },
    models::{TemperatureReading, Trackable, TruckDriverCard, TruckOdometerReading, TruckSpeed},
};
use vehicle_management_service::{
    apis::{trucks_api::CreateTruckSpeedError, Error as ApiError},
    models::TruckDriveState,
};

/// Enumeration for Teltonika event handlers.
///
/// This enumeration is used to store the different Teltonika event handlers and allow inheritance-like behavior.
#[derive(Debug)]
pub enum TeltonikaEventHandlers<'a> {
    SpeedEventHandler((SpeedEventHandler, &'a str)),
    DriverOneCardEventHandler((DriverOneCardEventHandler, &'a str)),
    DriverOneDriveStateEventHandler((DriverOneDriveStateEventHandler, &'a str)),
    OdometerReadingEventHandler((OdometerReadingEventHandler, &'a str)),
    TemperatureSensorsReadingEventHandler((TemperatureSensorsReadingEventHandler, &'a str)),
}

impl<'a> TeltonikaEventHandlers<'a> {
    pub fn event_handlers(log_target: &str) -> Vec<TeltonikaEventHandlers> {
        vec![
            TeltonikaEventHandlers::SpeedEventHandler((SpeedEventHandler, log_target)),
            TeltonikaEventHandlers::DriverOneCardEventHandler((DriverOneCardEventHandler, log_target)),
            TeltonikaEventHandlers::DriverOneDriveStateEventHandler((DriverOneDriveStateEventHandler, log_target)),
            TeltonikaEventHandlers::OdometerReadingEventHandler((OdometerReadingEventHandler, log_target)),
            TeltonikaEventHandlers::TemperatureSensorsReadingEventHandler((
                TemperatureSensorsReadingEventHandler,
                log_target,
            )),
        ]
    }

    /// Sends a failed event to the appropriate handler.
    ///
    /// # Arguments
    /// * `event_data` - The event data to send
    /// * `trackable` - The trackable object associated with the event
    /// * `log_target` - The log target for the event
    pub async fn send_failed_event(
        &self,
        event_data: String,
        trackable: Trackable,
        log_target: &str,
    ) -> Result<(), String> {
        match self {
            TeltonikaEventHandlers::SpeedEventHandler((handler, _)) => {
                let data: TruckSpeed =
                    serde_json::from_str(&event_data).map_err(|e| format!("Failed to deserialize TruckSpeed: {e}"))?;
                handler
                    .send_event(&data, trackable, log_target)
                    .await
                    .map_err(|e: ApiError<CreateTruckSpeedError>| format!("{e:?}"))
            }
            TeltonikaEventHandlers::DriverOneCardEventHandler((handler, _)) => {
                let data: TruckDriverCard = serde_json::from_str(&event_data)
                    .map_err(|e| format!("Failed to deserialize TruckDriverCard: {e}"))?;
                handler
                    .send_event(&data, trackable, log_target)
                    .await
                    .map_err(|e| format!("{e:?}"))
            }
            TeltonikaEventHandlers::DriverOneDriveStateEventHandler((handler, _)) => {
                let data: TruckDriveState = serde_json::from_str(&event_data)
                    .map_err(|e| format!("Failed to deserialize TruckDriveState: {e}"))?;
                handler
                    .send_event(&data, trackable, log_target)
                    .await
                    .map_err(|e: ApiError<CreateDriveStateError>| format!("{e:?}"))
            }
            TeltonikaEventHandlers::OdometerReadingEventHandler((handler, _)) => {
                let data: TruckOdometerReading = serde_json::from_str(&event_data)
                    .map_err(|e| format!("Failed to deserialize TruckOdometerReading: {e}"))?;
                handler
                    .send_event(&data, trackable, log_target)
                    .await
                    .map_err(|e: ApiError<CreateTruckOdometerReadingError>| format!("{e:?}"))
            }
            TeltonikaEventHandlers::TemperatureSensorsReadingEventHandler((handler, _)) => {
                let data: Vec<TemperatureReading> = serde_json::from_str(&event_data)
                    .map_err(|e| format!("Failed to deserialize TruckTemperatureSensorsReadings: {e}"))?;
                handler
                    .send_event(&data, trackable, log_target)
                    .await
                    .map_err(|e: ApiError<CreateTemperatureReadingError>| format!("{e:?}"))
            }
        }
    }

    pub fn require_all_events(&self) -> bool {
        match self {
            TeltonikaEventHandlers::SpeedEventHandler((handler, _)) => handler.require_all_events(),
            TeltonikaEventHandlers::DriverOneCardEventHandler((handler, _)) => handler.require_all_events(),
            TeltonikaEventHandlers::DriverOneDriveStateEventHandler((handler, _)) => handler.require_all_events(),
            TeltonikaEventHandlers::OdometerReadingEventHandler((handler, _)) => handler.require_all_events(),
            TeltonikaEventHandlers::TemperatureSensorsReadingEventHandler((handler, _)) => handler.require_all_events(),
        }
    }
    /// Gets the event ID for the handler.
    pub fn get_event_ids(&self, listener: &Listener) -> Vec<u16> {
        match self {
            TeltonikaEventHandlers::SpeedEventHandler((handler, _)) => handler.get_event_ids(listener),
            TeltonikaEventHandlers::DriverOneCardEventHandler((handler, _)) => handler.get_event_ids(listener),
            TeltonikaEventHandlers::DriverOneDriveStateEventHandler((handler, _)) => handler.get_event_ids(listener),
            TeltonikaEventHandlers::OdometerReadingEventHandler((handler, _)) => handler.get_event_ids(listener),
            TeltonikaEventHandlers::TemperatureSensorsReadingEventHandler((handler, _)) => {
                handler.get_event_ids(listener)
            }
        }
    }

    /// Gets the name of the event handler.
    ///
    /// # Returns
    /// The name of the event handler.
    pub fn get_event_handler_name(&self) -> String {
        match self {
            TeltonikaEventHandlers::SpeedEventHandler((handler, _)) => handler.get_event_handler_name(),
            TeltonikaEventHandlers::DriverOneCardEventHandler((handler, _)) => handler.get_event_handler_name(),
            TeltonikaEventHandlers::DriverOneDriveStateEventHandler((handler, _)) => handler.get_event_handler_name(),
            TeltonikaEventHandlers::OdometerReadingEventHandler((handler, _)) => handler.get_event_handler_name(),
            TeltonikaEventHandlers::TemperatureSensorsReadingEventHandler((handler, _)) => {
                handler.get_event_handler_name()
            }
        }
    }

    /// Gets the trigger event ID for the handler.
    pub fn get_trigger_event_ids(&self) -> Vec<u16> {
        match self {
            TeltonikaEventHandlers::SpeedEventHandler((handler, _)) => handler.get_trigger_event_ids(),
            TeltonikaEventHandlers::DriverOneCardEventHandler((handler, _)) => handler.get_trigger_event_ids(),
            TeltonikaEventHandlers::DriverOneDriveStateEventHandler((handler, _)) => handler.get_trigger_event_ids(),
            TeltonikaEventHandlers::OdometerReadingEventHandler((handler, _)) => handler.get_trigger_event_ids(),
            TeltonikaEventHandlers::TemperatureSensorsReadingEventHandler((handler, _)) => {
                handler.get_trigger_event_ids()
            }
        }
    }

    /// Handles a Teltonika event.
    pub async fn handle_events(
        &self,
        trigger_event_id: u16,
        events: Vec<&AVLEventIO>,
        timestamp: i64,
        imei: String,
        trackable: Option<Trackable>,
        listener: &Listener,
        database_pool: Pool<MySql>,
    ) {
        match self {
            TeltonikaEventHandlers::SpeedEventHandler((handler, log_target)) => {
                handler
                    .handle_events(
                        trigger_event_id,
                        events,
                        timestamp,
                        imei,
                        trackable,
                        log_target,
                        listener,
                        database_pool.clone(),
                    )
                    .await
            }
            TeltonikaEventHandlers::DriverOneCardEventHandler((handler, log_target)) => {
                handler
                    .handle_events(
                        trigger_event_id,
                        events,
                        timestamp,
                        imei,
                        trackable,
                        log_target,
                        listener,
                        database_pool.clone(),
                    )
                    .await
            }
            TeltonikaEventHandlers::DriverOneDriveStateEventHandler((handler, log_target)) => {
                handler
                    .handle_events(
                        trigger_event_id,
                        events,
                        timestamp,
                        imei,
                        trackable,
                        log_target,
                        listener,
                        database_pool.clone(),
                    )
                    .await
            }
            TeltonikaEventHandlers::OdometerReadingEventHandler((handler, log_target)) => {
                handler
                    .handle_events(
                        trigger_event_id,
                        events,
                        timestamp,
                        imei,
                        trackable,
                        log_target,
                        listener,
                        database_pool.clone(),
                    )
                    .await
            }
            TeltonikaEventHandlers::TemperatureSensorsReadingEventHandler((handler, log_target)) => {
                handler
                    .handle_events(
                        trigger_event_id,
                        events,
                        timestamp,
                        imei,
                        trackable,
                        log_target,
                        listener,
                        database_pool.clone(),
                    )
                    .await
            }
        }
    }
}

/// Trait for handling Teltonika events.
///
/// This trait is used to handle Teltonika events. It provides methods for handling events, sending events to the API and caching events.
///
/// # Type parameters
/// * `T` - The type of the event data to send to the API or Cache.
/// * `E` - The type of the error that can occur when sending the event to the API.
pub trait TeltonikaEventHandler<T, E>
where
    T: Serialize + for<'a> Deserialize<'a> + Clone + Debug,
    E: Debug,
    Vec<T>: Serialize + for<'a> Deserialize<'a> + Clone + Debug,
    Self: std::fmt::Debug,
{
    fn require_all_events(&self) -> bool {
        true
    }
    /// Gets the event ID for the handler.
    fn get_event_ids(&self, listener: &Listener) -> Vec<u16>;

    /// Gets the trigger event ID for the handler.
    ///
    /// If the trigger event ID is not the one that has triggered the record being processed (e.g. the records triggered event ID is 195 or 0 and the trigger event ID of the handler is 196), the record will be ignored.
    fn get_trigger_event_ids(&self) -> Vec<u16> {
        vec![]
    }

    /// Handles incoming Teltonika events.
    ///
    /// This method will process the event data, send it to the API and cache it if sending fails or truck id is not yet known.
    ///
    /// # Arguments
    /// * `trigger_event_id` - The trigger event ID of the [nom_teltonika::AVLRecord].
    /// * `events` - The Teltonika events to handle.
    /// * `timestamp` - The timestamp of the event.
    /// * `truck_id` - The truck ID of the event.
    /// * `base_cache_path` - The base path to the cache directory.
    /// * `log_target` - The log target to use for logging in format `imei - worker_id`.
    /// * 'listener' - Listener.
    async fn handle_events(
        &self,
        trigger_event_id: u16,
        events: Vec<&AVLEventIO>,
        timestamp: i64,
        imei: String,
        trackable: Option<Trackable>,
        log_target: &str,
        listener: &Listener,
        database_pool: Pool<MySql>,
    ) {
        let failed_events_handler = FailedEventsHandler::new(database_pool.clone());

        let event_data = self.process_event_data(trigger_event_id, &events, timestamp, log_target, listener);
        if event_data.is_none() {
            debug!(target: &log_target, "No event data to handle for {self:?}");
            return;
        }
        let event_data = event_data.unwrap();
        if let Some(ref trackable) = trackable {
            debug!(target: log_target, "[{self:?}] handling  event for {}: {}", trackable.trackable_type, trackable.id);
            let send_event_result = self.send_event(&event_data, trackable.clone(), log_target).await;
            if let Err(err) = send_event_result {
                error!(target: log_target, "Failed to send event for trackable {}: {err:?}. Persisting so it can be retried later", trackable.id);

                failed_events_handler
                    .persist_event(
                        imei.clone(),
                        FailedEvent {
                            id: None,
                            handler_name: self.get_event_handler_name(),
                            timestamp: timestamp,
                            event_data: serde_json::to_string(&event_data).unwrap(),
                            imei: imei.clone(),
                        },
                    )
                    .await
                    .expect("Failed to persist failed event");
            }
        } else {
            debug!(target: log_target, "Failed to send event for unknown truck: {}. Persisting so it can be retried later", imei);

            failed_events_handler
                .persist_event(
                    imei.clone(),
                    FailedEvent {
                        id: None,
                        handler_name: self.get_event_handler_name(),
                        timestamp: timestamp,
                        event_data: serde_json::to_string(&event_data).unwrap(),
                        imei: imei.clone(),
                    },
                )
                .await
                .expect("Failed to persist event for unknown truck");
        };
    }

    /// Sends the event data to the API.
    ///
    /// # Arguments
    /// * `event_data` - The event data to send.
    /// * `truck_id` - The truck ID of the event.
    /// * `log_target` - The log target to use for logging in format `imei - worker_id`.
    async fn send_event(&self, event_data: &T, trackable: Trackable, log_target: &str) -> Result<(), E>;

    /// Returns the name of the event handler.
    ///
    /// # Returns
    /// * The name of the event handler.
    fn get_event_handler_name(&self) -> String;

    /// Processes the event data.
    ///
    /// # Arguments
    /// * `trigger_event_id` - The trigger event ID of the [nom_teltonika::AVLRecord].
    /// * `events` - The Teltonika events data to process.
    /// * `truck_id` - The truck ID of the event.
    /// * `timestamp` - The timestamp of the event.
    /// * `log_target` - The log target to use for logging in format `imei - worker_id`.
    /// * 'listener' - Listener.
    ///
    /// # Returns
    /// * The processed event data.
    fn process_event_data(
        &self,
        trigger_event_id: u16,
        events: &Vec<&AVLEventIO>,
        timestamp: i64,
        log_target: &str,
        listener: &Listener,
    ) -> Option<T>;
}
