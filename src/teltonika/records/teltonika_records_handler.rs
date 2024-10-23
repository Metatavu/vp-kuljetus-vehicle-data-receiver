use std::path::{Path, PathBuf};

use crate::{
    telematics_cache::Cacheable,
    teltonika::{
        avl_event_io_value_to_u8,
        events::{
            DriverOneCardIdEventHandler, DriverOneDriveStateEventHandler, SpeedEventHandler,
            TeltonikaEventHandlers,
        },
        DRIVER_ONE_CARD_PRESENCE_EVENT_ID,
    },
    utils::get_vehicle_management_api_config,
    Message,
};
use chrono::{DateTime, Utc};
use log::debug;
use nom_teltonika::{AVLEventIO, AVLRecord};
use tokio::sync::mpsc::Sender;
use vehicle_management_service::{
    apis::trucks_api::CreateTruckLocationParams, models::TruckLocation,
};

use super::TeltonikaVinHandler;

/// Handler for Teltonika records.
pub struct TeltonikaRecordsHandler {
    base_cache_path: Box<Path>,
    truck_id: Option<String>,
    // event_handlers: Vec<TeltonikaEventHandlers>,
    imei: String,
    tx: Sender<Message>,
}

impl TeltonikaRecordsHandler {
    /// Creates a new [TeltonikaRecordsHandler].
    // pub fn new(
    //     base_cache_path: &Path,
    //     truck_id: Option<String>,
    //     imei: String,
    //     tx: Sender<Message>,
    // ) -> Self {
    //     TeltonikaRecordsHandler {
    //         base_cache_path: base_cache_path.into(),
    //         truck_id,
    //         event_handlers: vec![
    //             TeltonikaEventHandlers::SpeedEventHandler((SpeedEventHandler, imei.clone())),
    //             TeltonikaEventHandlers::DriverOneCardIdEventHandler((
    //                 DriverOneCardIdEventHandler,
    //                 imei.clone(),
    //             )),
    //             TeltonikaEventHandlers::DriverOneDriveStateEventHandler((
    //                 DriverOneDriveStateEventHandler,
    //                 imei.clone(),
    //             )),
    //         ],
    //         imei,
    //         tx,
    //     }
    // }

    fn event_handlers(log_target: &str) -> Vec<TeltonikaEventHandlers> {
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

    /// Gets the base cache path for the handler.
    #[cfg(test)]
    pub fn get_base_cache_path(&self) -> &Path {
        self.base_cache_path.as_ref()
    }
    /// Gets the truck VIN from a list of Teltonika [AVLRecord]s.
    ///
    /// This method will iterate over the records and find the VIN parts. If all three parts are found, they will be combined into a single VIN according to Teltonika specification.
    /// First VIN part has id 233, second 234 and third 235.
    ///
    /// # Arguments
    /// * `teltonika_records` - The list of [AVLRecord]s to get the VIN from.
    ///
    /// # Returns
    /// * The combined VIN if all three parts are found, otherwise None.
    pub fn get_truck_vin_from_records(teltonika_records: &Vec<AVLRecord>) -> Option<String> {
        let mut teltonika_vin = TeltonikaVinHandler::new();

        for record in teltonika_records.iter() {
            for event in record.io_events.iter() {
                if teltonika_vin
                    .get_teltonika_vin_event_ids()
                    .contains(&event.id)
                {
                    match &event.id {
                        233 => teltonika_vin.set_part_1(&event.value),
                        234 => teltonika_vin.set_part_2(&event.value),
                        235 => teltonika_vin.set_part_3(&event.value),
                        _ => (),
                    }
                }
            }
            // If we have all three parts, we can break the loop
            if teltonika_vin.get_is_complete() {
                break;
            }
        }

        return teltonika_vin.get_vin();
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
    pub async fn handle_records(
        teltonika_records: Vec<AVLRecord>,
        truck_id: Option<String>,
        base_cache_path: PathBuf,
        log_target: String,
    ) {
        for record in teltonika_records.iter() {
            Self::handle_record(
                record,
                truck_id.clone(),
                base_cache_path.clone(),
                &log_target,
            )
            .await;
        }
    }

    /// Handles a single Teltonika [AVLRecord].
    ///
    /// This method will iterate over the known event handlers and pass appropriate events to them.
    pub async fn handle_record(
        record: &AVLRecord,
        truck_id: Option<String>,
        base_cache_path: PathBuf,
        log_target: &str,
    ) {
        Self::handle_record_location(
            record,
            truck_id.clone(),
            log_target,
            base_cache_path.clone(),
        )
        .await;
        debug!(target: log_target, "Record trigger event ID: {}", record.trigger_event_id);
        for handler in Self::event_handlers(log_target).iter() {
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
                    truck_id.clone(),
                    base_cache_path.clone(),
                )
                .await;
        }
    }
    /// Purges the cache if Truck ID is known.
    pub async fn purge_cache(truck_id: String, log_target: &str, base_cache_path: Box<Path>) {
        Self::purge_location_cache(truck_id.clone(), log_target, base_cache_path.clone()).await;

        for handler in Self::event_handlers(log_target).iter() {
            handler
                .purge_cache(truck_id.clone(), base_cache_path.clone())
                .await;
        }
    }

    /// Handles a Teltonika [AVLRecord] location.
    ///
    /// Locations are separate from other events and are handled differently.
    /// This method will create a [CreateTruckLocationRequest] from the record and send it to the Vehicle Management Service or store in cache if truck ID is not yet known.
    async fn handle_record_location(
        record: &AVLRecord,
        truck_id: Option<String>,
        log_target: &str,
        base_cache_path: PathBuf,
    ) {
        let location_data = TruckLocation::from_teltonika_record(record).unwrap();
        if let Some(truck_id) = truck_id.clone() {
            debug!(target: log_target, "Handling location for truck: {}", truck_id);
            let result = vehicle_management_service::apis::trucks_api::create_truck_location(
                &get_vehicle_management_api_config(),
                CreateTruckLocationParams {
                    truck_id,
                    truck_location: location_data.clone(),
                },
            )
            .await;
            if let Err(e) = result {
                debug!(target: log_target,
                    "Error sending location: {:?}. Caching it for further use.",
                    e
                );
                location_data
                    .write_to_file(base_cache_path.to_str().unwrap())
                    .expect("Error caching location");
            }
        } else {
            debug!(target: log_target, "Caching location for yet unknown truck");
            location_data
                .write_to_file(base_cache_path.to_str().unwrap())
                .expect("Error caching location");
        }
    }

    /// Purges the location cache.
    async fn purge_location_cache(truck_id: String, log_target: &str, base_cache_path: Box<Path>) {
        let cache = TruckLocation::read_from_file(base_cache_path.to_str().unwrap());
        let mut failed_locations = Vec::new();

        for cached_location in cache.iter() {
            let result = vehicle_management_service::apis::trucks_api::create_truck_location(
                &get_vehicle_management_api_config(),
                CreateTruckLocationParams {
                    truck_id: truck_id.clone(),
                    truck_location: cached_location.clone(),
                },
            )
            .await;
            if let Err(e) = result {
                debug!(target: log_target,
                    "Error sending location: {:?}. Caching it for further use.",
                    e
                );
                failed_locations.push(cached_location.clone());
            }
        }
        let successful_locations_count = cache.len() - failed_locations.len();
        debug!(target: log_target,
            "Purged location cache of {} locations. {} failed to send.",
            successful_locations_count,
            failed_locations.len()
        );
        TruckLocation::clear_cache(base_cache_path.to_str().unwrap());
        for failed_location in failed_locations.iter() {
            failed_location
                .write_to_file(base_cache_path.to_str().unwrap())
                .expect("Error caching location");
        }
    }
}

/// Implementation of [Cacheable] for [CreateTruckLocationRequest].
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
