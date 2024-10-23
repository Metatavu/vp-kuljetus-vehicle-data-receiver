use super::{
    driver_one_card_id_event_handler, driver_one_drive_state_event_handler, speed_event_handler,
};
use crate::{
    telematics_cache::Cacheable,
    teltonika::events::{
        DriverOneCardIdEventHandler, DriverOneDriveStateEventHandler, SpeedEventHandler,
    },
};
use log::{debug, error};
use nom_teltonika::AVLEventIO;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, path::PathBuf};

/// Enumeration for Teltonika event handlers.
///
/// This enumeration is used to store the different Teltonika event handlers and allow inheritance-like behavior.
pub enum TeltonikaEventHandlers<'a> {
    SpeedEventHandler((speed_event_handler::SpeedEventHandler, &'a str)),
    DriverOneCardIdEventHandler(
        (
            driver_one_card_id_event_handler::DriverOneCardIdEventHandler,
            &'a str,
        ),
    ),
    DriverOneDriveStateEventHandler(
        (
            driver_one_drive_state_event_handler::DriverOneDriveStateEventHandler,
            &'a str,
        ),
    ),
}

impl<'a> TeltonikaEventHandlers<'a> {
    pub fn event_handlers(log_target: &str) -> Vec<TeltonikaEventHandlers> {
        vec![
            TeltonikaEventHandlers::SpeedEventHandler((SpeedEventHandler, log_target)),
            TeltonikaEventHandlers::DriverOneCardIdEventHandler((
                DriverOneCardIdEventHandler,
                log_target,
            )),
            TeltonikaEventHandlers::DriverOneDriveStateEventHandler((
                DriverOneDriveStateEventHandler,
                log_target,
            )),
        ]
    }
    /// Gets the event ID for the handler.
    pub fn get_event_ids(&self) -> Vec<u16> {
        match self {
            TeltonikaEventHandlers::SpeedEventHandler((handler, _)) => handler.get_event_ids(),
            TeltonikaEventHandlers::DriverOneCardIdEventHandler((handler, _)) => {
                handler.get_event_ids()
            }
            TeltonikaEventHandlers::DriverOneDriveStateEventHandler((handler, _)) => {
                handler.get_event_ids()
            }
        }
    }

    /// Gets the trigger event ID for the handler.
    pub fn get_trigger_event_id(&self) -> Option<u16> {
        match self {
            TeltonikaEventHandlers::SpeedEventHandler((handler, _)) => {
                handler.get_trigger_event_id()
            }
            TeltonikaEventHandlers::DriverOneCardIdEventHandler((handler, _)) => {
                handler.get_trigger_event_id()
            }
            TeltonikaEventHandlers::DriverOneDriveStateEventHandler((handler, _)) => {
                handler.get_trigger_event_id()
            }
        }
    }

    /// Handles a Teltonika event.
    pub async fn handle_events(
        &self,
        trigger_event_id: u16,
        events: Vec<&AVLEventIO>,
        timestamp: i64,
        truck_id: Option<String>,
        base_cache_path: PathBuf,
    ) {
        match self {
            TeltonikaEventHandlers::SpeedEventHandler((handler, imei)) => {
                handler
                    .handle_events(
                        trigger_event_id,
                        events,
                        timestamp,
                        truck_id,
                        base_cache_path,
                        imei,
                    )
                    .await
            }
            TeltonikaEventHandlers::DriverOneCardIdEventHandler((handler, imei)) => {
                handler
                    .handle_events(
                        trigger_event_id,
                        events,
                        timestamp,
                        truck_id,
                        base_cache_path,
                        imei,
                    )
                    .await
            }
            TeltonikaEventHandlers::DriverOneDriveStateEventHandler((handler, imei)) => {
                handler
                    .handle_events(
                        trigger_event_id,
                        events,
                        timestamp,
                        truck_id,
                        base_cache_path,
                        imei,
                    )
                    .await
            }
        }
    }

    /// Purges the cache.
    pub async fn purge_cache(&self, truck_id: String, base_cache_path: PathBuf) {
        match self {
            TeltonikaEventHandlers::SpeedEventHandler((handler, imei)) => {
                handler.purge_cache(truck_id, base_cache_path, imei).await
            }
            TeltonikaEventHandlers::DriverOneCardIdEventHandler((handler, imei)) => {
                handler.purge_cache(truck_id, base_cache_path, imei).await
            }
            TeltonikaEventHandlers::DriverOneDriveStateEventHandler((handler, imei)) => {
                handler.purge_cache(truck_id, base_cache_path, imei).await
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
    T: Cacheable + Serialize + for<'a> Deserialize<'a> + Clone + Debug,
    E: Debug,
{
    /// Gets the event ID for the handler.
    fn get_event_ids(&self) -> Vec<u16>;

    /// Gets the trigger event ID for the handler.
    ///
    /// If the trigger event ID is not the one that has triggered the record being processed (e.g. the records triggered event ID is 195 or 0 and the trigger event ID of the handler is 196), the record will be ignored.
    fn get_trigger_event_id(&self) -> Option<u16> {
        None
    }

    /// Handles a Teltonika event.
    ///
    /// This method will process the event data, send it to the API and cache it if sending fails or truck id is not yet known.
    ///
    /// # Arguments
    /// * `event` - The Teltonika event to handle.
    /// * `timestamp` - The timestamp of the event.
    /// * `truck_id` - The truck ID of the event.
    /// * `base_cache_path` - The base path to the cache directory.
    /// * `imei` - The IMEI of the device.
    async fn handle_events(
        &self,
        trigger_event_id: u16,
        events: Vec<&AVLEventIO>,
        timestamp: i64,
        truck_id: Option<String>,
        base_cache_path: PathBuf,
        imei: &str,
    ) {
        let event_data = self.process_event_data(trigger_event_id, &events, timestamp, imei);
        if event_data.is_none() {
            return;
        }
        let event_data = event_data.unwrap();
        if let Some(truck_id) = truck_id {
            debug!(target: imei, "Handling event for truck: {}", truck_id);
            let send_event_result = self.send_event(&event_data, truck_id).await;
            if let Err(e) = send_event_result {
                error!(target: imei, "Error sending event: {:?}. Caching it for further use.", e);
                self.cache_event_data(event_data, base_cache_path);
            }
        } else {
            debug!(target: imei, "Caching event for yet unknown truck");
            self.cache_event_data(event_data, base_cache_path);
        };
    }

    /// Caches the event data.
    ///
    /// # Arguments
    /// * `event` - The Teltonika event to cache.
    /// * `timestamp` - The timestamp of the event.
    /// * `base_cache_path` - The base path to the cache directory.
    fn cache_event_data(&self, event: T, base_cache_path: PathBuf) {
        let cache_result = event.write_to_file(base_cache_path.to_owned().to_str().unwrap());
        if let Err(e) = cache_result {
            panic!("Error caching event: {:?}", e);
        }
    }

    /// Sends the event data to the API.
    ///
    /// # Arguments
    /// * `event_data` - The event data to send.
    /// * `truck_id` - The truck ID of the event.
    async fn send_event(&self, event_data: &T, truck_id: String) -> Result<(), E>;

    /// Processes the event data.
    ///
    /// # Arguments
    /// * `event` - The Teltonika event data to process.
    /// * `truck_id` - The truck ID of the event.
    /// * `timestamp` - The timestamp of the event.
    /// * `imei` - The IMEI of the device.
    ///
    /// # Returns
    /// * The processed event data.
    fn process_event_data(
        &self,
        trigger_event_id: u16,
        events: &Vec<&AVLEventIO>,
        timestamp: i64,
        imei: &str,
    ) -> Option<T>;

    /// Purges the cache.
    ///
    /// # Arguments
    /// * `truck_id` - The truck ID to purge the cache for.
    /// * `base_cache_path` - The base path to the cache directory.
    /// * `imei` - The IMEI of the device.
    async fn purge_cache(&self, truck_id: String, base_cache_path: PathBuf, imei: &str) {
        let cache = T::read_from_file(base_cache_path.to_str().unwrap());
        let mut failed_events: Vec<T> = Vec::new();

        let event_ids = self
            .get_event_ids()
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        debug!(target: imei,
            "Purging cache of {} events for event ids: {}",
            cache.len(),
            event_ids
        );

        for cached_event in cache.iter() {
            let sent_event = self.send_event(cached_event, truck_id.clone()).await;
            if let Err(err) = sent_event {
                debug!(target: imei,
                    "Failed to send event: {:?}. Adding it to failed events.",
                    err
                );
                failed_events.push(cached_event.clone());
            }
        }
        let successful_events_count = cache.len() - failed_events.len();
        debug!(target: imei,
            "Purged {} events for event ids: {} from cache with {} failures",
            successful_events_count,
            event_ids,
            failed_events.len()
        );
        T::clear_cache(base_cache_path.to_str().unwrap());

        for failed_event in failed_events.iter() {
            failed_event
                .write_to_file(base_cache_path.to_owned().to_str().unwrap())
                .expect("Failed to write cache");
        }
    }
}
