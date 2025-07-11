use std::path::PathBuf;

use crate::{
    telematics_cache::Cacheable, teltonika::events::TeltonikaEventHandlers, utils::get_vehicle_management_api_config,
    Listener,
};
use log::debug;
use nom_teltonika::{AVLEventIO, AVLRecord};
use vehicle_management_service::{
    apis::trucks_api::CreateTruckLocationParams,
    models::{Trackable, TruckLocation},
};

/// Handler for Teltonika records.
pub struct TeltonikaRecordsHandler {
    log_target: String,
    trackable: Option<Trackable>,
    base_cache_path: PathBuf,
}

impl TeltonikaRecordsHandler {
    pub fn new(log_target: String, trackable: Option<Trackable>, base_cache_path: PathBuf) -> Self {
        TeltonikaRecordsHandler {
            log_target,
            trackable,
            base_cache_path,
        }
    }

    /// Gets the base cache path.
    #[cfg(test)]
    pub fn base_cache_path(&self) -> &std::path::Path {
        self.base_cache_path.as_path()
    }

    /// Handles a list of Teltonika [AVLRecord]s.
    ///
    /// # Arguments
    /// * `teltonika_records` - The list of [AVLRecord]s to handle.
    pub async fn handle_records(&self, teltonika_records: Vec<AVLRecord>, listener: &Listener) {
        for record in teltonika_records.iter() {
            self.handle_record(record, listener).await;
        }
    }

    /// Handles a single Teltonika [AVLRecord].
    ///
    /// This method will iterate over the known event handlers and pass appropriate events to them.
    ///
    /// # Arguments
    /// * `record` - The [AVLRecord] to handle.
    pub async fn handle_record(&self, record: &AVLRecord, listener: &Listener) {
        self.handle_record_location(record).await;
        let trigger_event = record
            .io_events
            .iter()
            .find(|event| event.id == record.trigger_event_id);
        debug!(target: &self.log_target, "Record trigger event: {:?}", trigger_event);
        debug!(target: &self.log_target, "Record trigger event id: {:?}", record.trigger_event_id);

        for handler in TeltonikaEventHandlers::event_handlers(&self.log_target).iter() {
            let trigger_event_ids = handler.get_trigger_event_ids();
            if !trigger_event_ids.is_empty() && !trigger_event_ids.contains(&record.trigger_event_id) {
                continue;
            }
            let events = handler
                .get_event_ids(listener)
                .iter()
                .map(|id| {
                    record
                        .io_events
                        .iter()
                        .filter(|event| event.id == *id)
                        .collect::<Vec<&AVLEventIO>>()
                })
                .flatten()
                .collect::<Vec<&AVLEventIO>>();
            // If we don't have any events we skip the handler
            if events.is_empty() {
                continue;
            }
            // If the handler requires all events and we don't have all of them we skip the handler
            if handler.require_all_events() && handler.get_event_ids(listener).len() != events.len() {
                continue;
            }
            handler
                .handle_events(
                    record.trigger_event_id,
                    events,
                    record.timestamp.timestamp(),
                    self.trackable.clone(),
                    self.base_cache_path.clone(),
                    listener,
                )
                .await;
        }
    }

    /// Handles a Teltonika [AVLRecord] location.
    ///
    /// Locations are separate from other events and are handled differently.
    /// This function will create a [TruckLocation] from the record and send it to the Vehicle Management Service or store in cache if truck ID is not yet known.
    ///
    /// # Arguments
    /// * `record` - The [AVLRecord] to handle the location for.
    async fn handle_record_location(&self, record: &AVLRecord) {
        let location_data = TruckLocation::from_teltonika_record(record).unwrap();
        if let Some(trackable) = self.trackable.clone() {
            debug!(target: &self.log_target, "Handling location for trackable: {}", trackable.id);
            let result = vehicle_management_service::apis::trucks_api::create_truck_location(
                &get_vehicle_management_api_config(),
                CreateTruckLocationParams {
                    truck_id: trackable.id.to_string(),
                    truck_location: location_data.clone(),
                },
            )
            .await;
            if let Err(e) = result {
                debug!(target: &self.log_target,
                    "Error sending location: {:?}. Caching it for further use.",
                    e
                );
                location_data
                    .write_to_file(self.base_cache_path.clone())
                    .expect("Error caching location");
            }
        } else {
            debug!(target: &self.log_target, "Caching location for yet unknown truck");
            location_data
                .write_to_file(self.base_cache_path.clone())
                .expect("Error caching location");
        }
    }
}

/// Implementation of [Cacheable] for [TruckLocation].
impl Cacheable for TruckLocation {
    fn get_file_path() -> String
    where
        Self: Sized,
    {
        String::from("truck_location_cache.json")
    }

    fn from_teltonika_record(record: &AVLRecord) -> Option<Self>
    where
        Self: Sized,
    {
        Some(TruckLocation {
            id: None,
            latitude: record.latitude,
            longitude: record.longitude,
            heading: record.angle as f64,
            timestamp: record.timestamp.timestamp(),
        })
    }
}

/// Implementation of [Cacheable] for [TruckLocation].
impl Cacheable for Vec<TruckLocation> {
    fn get_file_path() -> String
    where
        Self: Sized,
    {
        String::from("truck_location_cache.json")
    }
}
