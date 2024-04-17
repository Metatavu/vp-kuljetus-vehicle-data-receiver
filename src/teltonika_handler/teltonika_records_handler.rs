use std::path::Path;
use log::debug;
use nom_teltonika::AVLRecord;
use super::{speed_event_handler::SpeedEventHandler, teltonika_event_handlers::TeltonikaEventHandlers, teltonika_vin_handler::TeltonikaVinHandler};

/// Handler for Teltonika records.
pub struct TeltonikaRecordsHandler {
  base_cache_path: Box<Path>,
  truck_id: Option<String>,
  event_handlers: Vec<TeltonikaEventHandlers>,
}

impl TeltonikaRecordsHandler {

  /// Creates a new [TeltonikaRecordsHandler].
  pub fn new(base_cache_path: &Path, truck_id: Option<String>) -> Self {
    TeltonikaRecordsHandler {
      base_cache_path: base_cache_path.into(),
      truck_id,
      event_handlers: vec![
        TeltonikaEventHandlers::SpeedEventHandler(SpeedEventHandler {}),
      ]
    }
  }

  /// Gets the base cache path for the handler.
  #[cfg(test)]
  pub fn get_base_cache_path(&self) -> &Path {
    self.base_cache_path.as_ref()
  }

  /// Sets the truck ID for the handler.
  ///
  /// # Arguments
  /// * `truck_id` - The truck ID to set.
  pub fn set_truck_id(&mut self, truck_id: Option<String>) {
    self.truck_id = truck_id;
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
  pub fn get_truck_vin_from_records(&self, teltonika_records: &Vec<AVLRecord>) -> Option<String> {
    let mut teltonika_vin = TeltonikaVinHandler::new();

    for record in teltonika_records.iter() {
      for event in record.io_events.iter() {
        if teltonika_vin.get_teltonika_vin_event_ids().contains(&event.id) {
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

  /// Handles a list of Teltonika [AVLRecord]s.
  pub async fn handle_records(&self, teltonika_records: Vec<AVLRecord>) {
    for record in teltonika_records.iter() {
      self.handle_record(record).await;
    }
  }

  /// Handles a single Teltonika [AVLRecord].
  ///
  /// This method will iterate over the IO events in the record and call the appropriate handler for each event.
  pub async fn handle_record(&self, record: &AVLRecord) {
    for event in record.io_events.iter() {
      if let Some(handler) = self.get_event_handler(event.id) {
        handler
          .handle_event(&event, record.timestamp.timestamp(), self.truck_id.clone(), self.base_cache_path.clone())
          .await;
      } else {
        debug!("No handler found for event id: {}", event.id);
      }
    }
  }

  /// Purges the cache if Truck ID is known.
  pub async fn purge_cache(&self) {
    if self.truck_id.is_none() {
      return;
    }

    for handler in self.event_handlers.iter() {
      handler
        .purge_cache(self.truck_id.clone().unwrap(), self.base_cache_path.clone())
        .await;
    }
  }

  /// Gets the event handler for a specific event ID.
  ///
  /// # Arguments
  /// * `event_id` - The event ID to get the handler for.
  ///
  /// # Returns
  /// * The event handler if found, otherwise None.
  fn get_event_handler(&self, event_id: u16) -> Option<&TeltonikaEventHandlers> {
    for handler in self.event_handlers.iter() {
      if handler.get_event_id() == event_id {
        return Some(handler);
      }
    }
    return None;
  }
}