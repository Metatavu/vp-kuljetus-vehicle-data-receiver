use nom_teltonika::{AVLEventIO, AVLEventIOValue, AVLRecord};

mod speed_event_handler;

pub struct TeltonikaEventHandler {
  event_id: u16,
  handler: fn(AVLEventIO, i64, Option<String>),
}

const TELTONIKA_EVENT_HANDLERS: [TeltonikaEventHandler; 1] = [
  speed_event_handler::SPEED_EVENT_HANDLER,
];

const TELTONIKA_VIN_EVENT_IDS: [u16; 3] = [233, 234, 235];

pub struct TeltonikaRecordsHandler {
  cache_base_path: String,
}

impl TeltonikaRecordsHandler {
  /// Creates a new [TeltonikaRecordsHandler] with the given cache base path.
  pub fn new(cache_base_path: String) -> Self {
    TeltonikaRecordsHandler {
      cache_base_path,
    }
  }

  /// Gets the truck VIN from a list of Teltonika [AVLRecord]s.
  ///
  /// This method will iterate over the records and find the VIN parts. If all three parts are found, they will be combined into a single VIN according to Teltonika documentation.
  ///
  /// # Arguments
  /// * `teltonika_records` - The list of [AVLRecord]s to get the VIN from.
  ///
  /// # Returns
  /// * The combined VIN if all three parts are found, otherwise None.
  pub fn get_truck_vin_from_records(&self, teltonika_records: Vec<AVLRecord>) -> Option<String> {
    let mut vin_parts: Vec<AVLEventIO> = Vec::new();

    for record in teltonika_records.iter() {
      for event in record.io_events.iter() {
        if TELTONIKA_VIN_EVENT_IDS.contains(&event.id) && !vin_parts.iter().any(|vin_part| vin_part.id == event.id) {
          vin_parts.push(event.clone());
        }
      }
      if vin_parts.len() == 3 {
        break;
      }
    }
    if vin_parts.len() < 3 {
      return None;
    }

    vin_parts.sort_by(|a, b| a.id.cmp(&b.id));
    let combined_vin = vin_parts.iter().fold(Vec::new(), |mut vec, vin_part| {
      match &vin_part.value {
        AVLEventIOValue::U64(value) => {
          let mut bytes = value.to_be_bytes().to_vec();
          vec.append(&mut bytes);
        }
        AVLEventIOValue::U8(value) => {
          let mut bytes = value.to_be_bytes().to_vec();
          vec.append(&mut bytes);
        }
        _ => (),
      }

      return vec;
    });
    let actual_vin = String::from_utf8(combined_vin).unwrap();

    return Some(actual_vin);

  }

  /// Handles a list of Teltonika [AVLRecord]s.
  ///
  /// # Arguments
  /// * `teltonika_records` - The list of [AVLRecord]s to handle.
  /// * `truck_id` - The truck ID to associate with the records.
  pub fn handle_records(&self, teltonika_records: Vec<AVLRecord>, truck_id: Option<String>) {
    for record in teltonika_records.iter() {
      self.handle_record(record, truck_id.clone());
    }
  }

  /// Handles a single Teltonika [AVLRecord].
  ///
  /// This method will iterate over the [AVLEventIO]s in the record and call the appropriate handler for each event.
  /// If a handler is found for the event, it will be called with the event, the timestamp of the record and the truck_id.
  /// Handler will then either store the event in file system or send it to the server, depending whether `truck_id` is defined or not.
  ///
  /// # Arguments
  /// * `teltonika_record` - The [AVLRecord] to handle.
  /// * `truck_id` - The truck ID to associate with the record.
  pub fn handle_record(&self, teltonika_record: &AVLRecord, truck_id: Option<String>) {
    for event in teltonika_record.io_events.iter() {
      let handler = TELTONIKA_EVENT_HANDLERS.iter().find(|handler| event.id == handler.event_id);
      if let Some(handler) = handler {
        (handler.handler)(event.clone(), teltonika_record.timestamp.timestamp(), truck_id.clone());
      } else {
        println!("No handler found for event: {:?}", event);
      }
    }
  }
}