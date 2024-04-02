mod speed_event_handler;

use std::path::Path;

use nom_teltonika::{AVLEventIOValue, AVLRecord, AVLEventIO};

struct TeltonikaEventHandler {
  event_id: u16,
  handler: fn(AVLEventIO, i64, Option<String>, Box<Path>),
  purge: fn(String, Box<Path>),
}

const TELTONIKA_VIN_EVENT_IDS: [u16; 3] = [233, 234, 235];

const TELTONIKA_CACHEABLE_HANDLERS: [TeltonikaEventHandler; 1]= [
  speed_event_handler::SPEED_EVENT_HANDLER,
];

fn avl_event_io_value_to_be_bytes(value: &AVLEventIOValue) -> Vec<u8> {
  match value {
    AVLEventIOValue::U64(value) => value.to_be_bytes().to_vec(),
    AVLEventIOValue::U8(value) => value.to_be_bytes().to_vec(),
    _ => Vec::new(),
  }
}

/// Struct to hold the binary parts of a Teltonika VIN
pub struct TeltonikaVin {
  part_1: Option<Vec<u8>>,
  part_2: Option<Vec<u8>>,
  part_3: Option<Vec<u8>>,
}

impl TeltonikaVin {
  pub fn new() -> Self {
    TeltonikaVin {
      part_1: None,
      part_2: None,
      part_3: None,
    }
  }

  /// Checks if all three parts of the VIN are present.
  pub fn get_is_complete(&self) -> bool {
    return self.part_1.is_some() && self.part_2.is_some() && self.part_3.is_some();
  }

  pub fn set_part_1(&mut self, value: &AVLEventIOValue) {
    if self.part_1.is_none() {
      self.part_1 = Some(avl_event_io_value_to_be_bytes(value));
    }
  }

  pub fn set_part_2(&mut self, value: &AVLEventIOValue) {
    if self.part_2.is_none() {
      self.part_2 = Some(avl_event_io_value_to_be_bytes(value));
    }
  }

  pub fn set_part_3(&mut self, value: &AVLEventIOValue) {
    if self.part_3.is_none() {
      self.part_3 = Some(avl_event_io_value_to_be_bytes(value));
    }
  }

  /// Combines the three binary parts of the VIN into the full string representation.
  pub fn get_vin(&mut self) -> Option<String> {
    if self.part_1.is_some() && self.part_2.is_some() && self.part_3.is_some() {
      let mut vin = Vec::new();
      vin.extend_from_slice(&self.part_1.clone().unwrap());
      vin.extend_from_slice(&self.part_2.clone().unwrap());
      vin.extend_from_slice(&self.part_3.clone().unwrap());
      return Some(String::from_utf8(vin).unwrap());
    }

    return None;
  }
}

pub struct TeltonikaRecordsHandler {
  cache_base_path: Box<Path>,
  truck_id: Option<String>,
}

impl TeltonikaRecordsHandler {
  /// Creates a new [TeltonikaRecordsHandler].
  pub fn new(cache_base_path: &Path, truck_id: Option<String>) -> Self {
    TeltonikaRecordsHandler { cache_base_path: cache_base_path.into(), truck_id: truck_id }
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
    let mut teltonika_vin = TeltonikaVin::new();

    for record in teltonika_records.iter() {
      for event in record.io_events.iter() {
        if TELTONIKA_VIN_EVENT_IDS.contains(&event.id) {
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
  pub fn handle_records(&self, teltonika_records: Vec<AVLRecord>) {
    for record in teltonika_records.iter() {
      self.handle_record(record);
    }
  }

  /// Handles a single Teltonika [AVLRecord].
  ///
  /// This method will iterate over the IO events in the record and call the appropriate handler for each event.
  pub fn handle_record(&self, record: &AVLRecord) {
    for event in record.io_events.iter() {
      for handler in TELTONIKA_CACHEABLE_HANDLERS.iter() {
        if handler.event_id == event.id {
          (handler.handler)(event.clone(), record.timestamp.timestamp(), self.truck_id.clone(), self.cache_base_path.clone());
        }
      }
    }
  }

  /// Purges the cache for the truck ID.
  pub fn purge_cache(&self) {
    for handler in TELTONIKA_CACHEABLE_HANDLERS.iter() {
      (handler.purge)(self.truck_id.clone().unwrap(), self.cache_base_path.clone());
    }
  }
}