use std::path::{Path, PathBuf};

use crate::{
    telematics_cache::Cacheable,
    teltonika::{
        avl_event_io_value_to_u8, events::TeltonikaEventHandlers, DRIVER_ONE_CARD_PRESENCE_EVENT_ID,
    },
    utils::get_vehicle_management_api_config,
};
use chrono::{DateTime, Utc};
use log::debug;
use nom_teltonika::{AVLEventIO, AVLRecord};
use vehicle_management_service::{
    apis::trucks_api::CreateTruckLocationParams, models::TruckLocation,
};

/// Handler for Teltonika records.
pub struct TeltonikaRecordsHandler {
    log_target: String,
    truck_id: Option<String>,
    base_cache_path: PathBuf,
}

impl TeltonikaRecordsHandler {
    pub fn new(log_target: String, truck_id: Option<String>, base_cache_path: PathBuf) -> Self {
        TeltonikaRecordsHandler {
            log_target,
            truck_id,
            base_cache_path,
        }
    }

    /// Gets the base cache path.
    pub fn base_cache_path(&self) -> &Path {
        self.base_cache_path.as_path()
    }

    /// Returns the driver one card presence from a list of Teltonika [AVLRecord]s.
    ///
    /// # Arguments
    /// * `teltonika_records` - The list of [AVLRecord]s to get the driver one card presence from.
    ///
    /// # Returns
    /// * Tuple where first value is the driver one card presence and second value is the latest [AVLRecord] with the driver one card presence event.
    pub fn get_driver_one_card_presence_from_records(
        teltonika_records: &mut Vec<AVLRecord>,
    ) -> Option<(bool, Option<DateTime<Utc>>)> {
        teltonika_records.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        let driver_one_card_presence_records: Vec<&AVLRecord> = teltonika_records
            .iter()
            .filter(|record| record.trigger_event_id == DRIVER_ONE_CARD_PRESENCE_EVENT_ID)
            .collect();
        if let Some(latest_record) = driver_one_card_presence_records.first() {
            let latest_event = latest_record
                .io_events
                .iter()
                .find(|event| event.id == DRIVER_ONE_CARD_PRESENCE_EVENT_ID);

            return match latest_event {
                Some(event) => Some((
                    avl_event_io_value_to_u8(&event.value) == 1,
                    Some(latest_record.timestamp),
                )),
                None => None,
            };
        }

        return None;
    }

    /// Handles a list of Teltonika [AVLRecord]s.
    ///
    /// # Arguments
    /// * `teltonika_records` - The list of [AVLRecord]s to handle.
    pub async fn handle_records(&self, teltonika_records: Vec<AVLRecord>) {
        for record in teltonika_records.iter() {
            self.handle_record(record).await;
        }
    }

    /// Handles a single Teltonika [AVLRecord].
    ///
    /// This method will iterate over the known event handlers and pass appropriate events to them.
    ///
    /// # Arguments
    /// * `record` - The [AVLRecord] to handle.
    pub async fn handle_record(&self, record: &AVLRecord) {
        self.handle_record_location(record).await;
        debug!(target: &self.log_target, "Record trigger event ID: {}", record.trigger_event_id);
        for handler in TeltonikaEventHandlers::event_handlers(&self.log_target).iter() {
            let trigger_event_id = handler.get_trigger_event_id();
            if trigger_event_id.is_some() && record.trigger_event_id != trigger_event_id.unwrap() {
                continue;
            }
            let events = handler
                .get_event_ids()
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
            // If we don't have any events or the number of events is not the same as the number of event IDs, we skip the handler
            if events.is_empty() || handler.get_event_ids().len() != events.len() {
                continue;
            }
            handler
                .handle_events(
                    record.trigger_event_id,
                    events,
                    record.timestamp.timestamp(),
                    self.truck_id.clone(),
                    self.base_cache_path.clone(),
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
        if let Some(truck_id) = self.truck_id.clone() {
            debug!(target: &self.log_target, "Handling location for truck: {}", truck_id);
            let result = vehicle_management_service::apis::trucks_api::create_truck_location(
                &get_vehicle_management_api_config(),
                CreateTruckLocationParams {
                    truck_id,
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
                    .write_to_file(&self.base_cache_path.to_str().unwrap())
                    .expect("Error caching location");
            }
        } else {
            debug!(target: &self.log_target, "Caching location for yet unknown truck");
            location_data
                .write_to_file(&self.base_cache_path.to_str().unwrap())
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

    fn from_teltonika_record(_: &AVLRecord) -> Option<Self>
    where
        Self: Sized,
    {
        None
    }
}
