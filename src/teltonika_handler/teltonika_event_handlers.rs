use super::{driver_one_card_id_event_handler, speed_event_handler};
use crate::telematics_cache::Cacheable;
use log::{debug, error};
use nom_teltonika::AVLEventIO;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, path::Path};

/// Enumeration for Teltonika event handlers.
///
/// This enumeration is used to store the different Teltonika event handlers and allow inheritance-like behavior.
pub enum TeltonikaEventHandlers {
    SpeedEventHandler(speed_event_handler::SpeedEventHandler),
    DriverOneCardIdEventHandler(driver_one_card_id_event_handler::DriverOneCardIdEventHandler),
}

impl TeltonikaEventHandlers {
    /// Gets the event ID for the handler.
    pub fn get_event_ids(&self) -> Vec<u16> {
        match self {
            TeltonikaEventHandlers::SpeedEventHandler(handler) => handler.get_event_ids(),
            TeltonikaEventHandlers::DriverOneCardIdEventHandler(handler) => handler.get_event_ids(),
        }
    }

    /// Handles a Teltonika event.
    pub async fn handle_events(
        &self,
        events: Vec<&AVLEventIO>,
        timestamp: i64,
        truck_id: Option<String>,
        base_cache_path: Box<Path>,
    ) {
        match self {
            TeltonikaEventHandlers::SpeedEventHandler(handler) => {
                handler
                    .handle_events(events, timestamp, truck_id, base_cache_path)
                    .await
            }
            TeltonikaEventHandlers::DriverOneCardIdEventHandler(handler) => {
                handler
                    .handle_events(events, timestamp, truck_id, base_cache_path)
                    .await
            }
        }
    }

    /// Purges the cache.
    pub async fn purge_cache(&self, truck_id: String, base_cache_path: Box<Path>) {
        match self {
            TeltonikaEventHandlers::SpeedEventHandler(handler) => {
                handler.purge_cache(truck_id, base_cache_path).await
            }
            TeltonikaEventHandlers::DriverOneCardIdEventHandler(handler) => {
                handler.purge_cache(truck_id, base_cache_path).await
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

    /// Handles a Teltonika event.
    ///
    /// This method will process the event data, send it to the API and cache it if sending fails or truck id is not yet known.
    ///
    /// # Arguments
    /// * `event` - The Teltonika event to handle.
    /// * `timestamp` - The timestamp of the event.
    /// * `truck_id` - The truck ID of the event.
    /// * `base_cache_path` - The base path to the cache directory.
    async fn handle_events(
        &self,
        events: Vec<&AVLEventIO>,
        timestamp: i64,
        truck_id: Option<String>,
        base_cache_path: Box<Path>,
    ) {
        let event_data = self.process_event_data(&events, timestamp);
        if let Some(truck_id) = truck_id {
            debug!("Handling event for truck: {}", truck_id);
            let send_event_result = self.send_event(&event_data, truck_id).await;
            if let Err(e) = send_event_result {
                error!("Error sending event: {:?}. Caching it for further use.", e);
                self.cache_event_data(event_data, base_cache_path);
            }
        } else {
            debug!("Caching event for yet unknown truck");
            self.cache_event_data(event_data, base_cache_path);
        };
    }

    /// Caches the event data.
    ///
    /// # Arguments
    /// * `event` - The Teltonika event to cache.
    /// * `timestamp` - The timestamp of the event.
    /// * `base_cache_path` - The base path to the cache directory.
    fn cache_event_data(&self, event: T, base_cache_path: Box<Path>) {
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
    ///
    /// # Returns
    /// * The processed event data.
    fn process_event_data(&self, events: &Vec<&AVLEventIO>, timestamp: i64) -> T;

    /// Purges the cache.
    ///
    /// # Arguments
    /// * `truck_id` - The truck ID to purge the cache for.
    /// * `base_cache_path` - The base path to the cache directory.
    async fn purge_cache(&self, truck_id: String, base_cache_path: Box<Path>) {
        let cache = T::read_from_file(base_cache_path.to_str().unwrap());
        let mut failed_events: Vec<T> = Vec::new();

        let event_ids = self
            .get_event_ids()
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        debug!(
            "Purging cache of {} events for event ids: {}",
            cache.len(),
            event_ids
        );

        for cached_event in cache.iter() {
            let sent_event = self.send_event(cached_event, truck_id.clone()).await;
            if let Err(err) = sent_event {
                debug!(
                    "Failed to send event: {:?}. Adding it to failed events.",
                    err
                );
                failed_events.push(cached_event.clone());
            }
        }
        let successful_events_count = cache.len() - failed_events.len();
        debug!(
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
