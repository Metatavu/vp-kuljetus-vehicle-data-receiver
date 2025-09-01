use crate::{
    failed_events::{FailedEvent, FailedEventError, FailedEventsHandler},
    teltonika::events::TeltonikaEventHandlers,
    utils::get_vehicle_management_api_config,
    Listener,
};
use futures::future::join_all;
use log::{debug, error, info, warn};
use nom_teltonika::{AVLEventIO, AVLRecord};
use sqlx::{MySql, Pool};
use vehicle_management_service::{
    apis::trucks_api::CreateTruckLocationParams,
    models::{Trackable, TruckLocation},
};

/// Handler for Teltonika records.
pub struct TeltonikaRecordsHandler {
    log_target: String,
    trackable: Trackable,
    imei: String,
}

impl TeltonikaRecordsHandler {
    pub fn new(log_target: String, trackable: Trackable, imei: String) -> Self {
        TeltonikaRecordsHandler {
            log_target,
            trackable,
            imei,
        }
    }

    /// Handles a list of Teltonika [AVLRecord]s.
    ///
    /// # Arguments
    /// * `teltonika_records` - The list of [AVLRecord]s to handle.
    pub async fn handle_records(
        &self,
        teltonika_records: Vec<AVLRecord>,
        listener: &Listener,
    ) -> Result<(), FailedEventError> {
        let mut failed_to_process = false;
        for task in teltonika_records.iter() {
            let result = self.handle_record(task, listener).await;
            if result.is_err() {
                failed_to_process = true;
            }
        }

        if failed_to_process {
            return Err(FailedEventError::FailedToSend);
        }

        Ok(())
    }

    /// Resends failed events.
    ///
    /// This method will attempt to resend them using the appropriate event handlers.
    ///
    /// # Arguments
    /// * `failed_event` - The failed event to handle.
    pub async fn handle_failed_event(&self, failed_event: FailedEvent) -> Result<u64, FailedEventError> {
        let event_id = failed_event.id.ok_or(FailedEventError::MissingId)?;

        if failed_event.handler_name == "location" {
            match self.send_failed_location(failed_event).await {
                Ok(()) => {
                    info!(target: &self.log_target, "reprocessed failed location event: {}", event_id);
                    Ok(event_id)
                }
                Err(e) => {
                    warn!(target: &self.log_target, "Failed to resend failed location event");
                    Err(FailedEventError::FailedToResend)
                }
            }
        } else {
            // Find the one matching handler; error if none.
            let handler = TeltonikaEventHandlers::event_handlers(&self.log_target)
                .into_iter()
                .find(|h| h.get_event_handler_name() == failed_event.handler_name)
                .ok_or_else(|| FailedEventError::HandlerNotFound(failed_event.handler_name.clone()))?;

            // Reprocess
            match handler
                .send_failed_event(
                    failed_event.event_data, // avoid unnecessary clone if possible
                    self.trackable.clone(),  // or pass Option upstream
                    &self.imei,
                )
                .await
            {
                Ok(()) => {
                    info!(target: &self.log_target, "reprocessed failed event: {}", event_id);
                    Ok(event_id)
                }
                Err(e) => {
                    warn!(target: &self.log_target, "Failed to resend failed event: {}", e);
                    Err(FailedEventError::FailedToResend)
                }
            }
        }
    }

    /// Sends a failed location event for reprocessing.
    ///
    /// # Arguments
    /// * `failed_event` - The failed event to process.
    async fn send_failed_location(&self, failed_event: FailedEvent) -> Result<(), String> {
        let event_data = failed_event.event_data.as_str();
        let trackable = self.trackable.clone();

        let location_data: TruckLocation = serde_json::from_str(&event_data).map_err(|e| format!("{e:?}"))?;

        vehicle_management_service::apis::trucks_api::create_truck_location(
            &get_vehicle_management_api_config(),
            CreateTruckLocationParams {
                truck_id: trackable.id.to_string(),
                truck_location: location_data.clone(),
            },
        )
        .await
        .map_err(|e| format!("{e:?}"))
    }

    /// Handles a single Teltonika [AVLRecord].
    ///
    /// This method will iterate over the known event handlers and pass appropriate events to them.
    ///
    /// # Arguments
    /// * `record` - The [AVLRecord] to handle.
    pub async fn handle_record(&self, record: &AVLRecord, listener: &Listener) -> Result<(), FailedEventError> {
        if *listener == Listener::TeltonikaFMC234 {
            debug!(target: &self.log_target, "Skipping location for {listener:?} listener as not yet implemented on backend")
        } else {
            self.handle_record_location(record).await;
        }
        let trigger_event = record
            .io_events
            .iter()
            .find(|event| event.id == record.trigger_event_id);
        debug!(target: &self.log_target, "Record trigger event: {:?}", trigger_event);
        debug!(target: &self.log_target, "Record trigger event id: {:?}", record.trigger_event_id);

        let mut failed_to_process = false;
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
            //debug!(target: &self.log_target, "{record:#?}");
            if events.is_empty() {
                debug!(target: &self.log_target, "No events found for handler: {handler:?}");
                continue;
            }
            // If the handler requires all events and we don't have all of them we skip the handler
            if handler.require_all_events() && handler.get_event_ids(listener).len() != events.len() {
                continue;
            }

            match handler
                .handle_events(
                    record.trigger_event_id,
                    events,
                    record.timestamp.timestamp(),
                    self.imei.clone(),
                    self.trackable.clone(),
                    listener,
                )
                .await
            {
                Ok(_) => {
                    debug!(target: &self.log_target, "Handler {handler:?} processed events successfully");
                }
                Err(e) => {
                    error!(target: &self.log_target, "Failed to handle events");
                    failed_to_process = true;
                    break;
                }
            };
        }

        if failed_to_process {
            return Err(FailedEventError::FailedToSend);
        }

        Ok(())
    }

    /// Handles a Teltonika [AVLRecord] location.
    ///
    /// Locations are separate from other events and are handled differently.
    /// This function will create a [TruckLocation] from the record and send it to the Vehicle Management Service or store in cache if truck ID is not yet known.
    ///
    /// # Arguments
    /// * `record` - The [AVLRecord] to handle the location for.
    async fn handle_record_location(&self, record: &AVLRecord) {
        let location_data = TruckLocation {
            id: None,
            latitude: record.latitude,
            longitude: record.longitude,
            heading: record.angle as f64,
            timestamp: record.timestamp.timestamp(),
        };

        debug!(target: &self.log_target, "Handling location for trackable: {}", self.trackable.id);
        let result = vehicle_management_service::apis::trucks_api::create_truck_location(
            &get_vehicle_management_api_config(),
            CreateTruckLocationParams {
                truck_id: self.trackable.id.to_string(),
                truck_location: location_data.clone(),
            },
        )
        .await;
        if let Err(e) = result {
            debug!(target: &self.log_target,
                "Failed to send location: {:?}. Persisting into database, so it can be retried later.",
                e
            );
        }
    }
}
