use nom_teltonika::{AVLEventIO, AVLEventIOValue, AVLRecord};

const TELTONIKA_VIN_EVENT_IDS: [u16; 3] = [233, 234, 235];

pub struct TeltonikaRecordsHandler {}

impl TeltonikaRecordsHandler {
  /// Creates a new [TeltonikaRecordsHandler].
  pub fn new() -> Self {
    TeltonikaRecordsHandler {}
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
  pub fn get_truck_vin_from_records(&self, teltonika_records: Vec<AVLRecord>) -> Option<String> {
    let mut vin_parts: Vec<AVLEventIO> = Vec::new();

    for record in teltonika_records.iter() {
      for event in record.io_events.iter() {
        // Check if the event is a VIN part and if it is not already in the list
        if TELTONIKA_VIN_EVENT_IDS.contains(&event.id) && !vin_parts.iter().any(|vin_part| vin_part.id == event.id) {
          vin_parts.push(event.clone());
        }
      }
      // If we have all three parts, we can break the loop
      if vin_parts.len() == 3 {
        break;
      }
    }

    // If we don't have all three parts, we can return None
    if vin_parts.len() < 3 {
      return None;
    }

    // Sort the VIN parts by ID and combine them into a single vector
    vin_parts.sort_by(|a, b| a.id.cmp(&b.id));
    let combined_vin = vin_parts.iter().fold(Vec::new(), |mut accumulator, vin_part| {
      match &vin_part.value {
        AVLEventIOValue::U64(value) => {
          let mut bytes = value.to_be_bytes().to_vec();
          accumulator.append(&mut bytes);
        }
        AVLEventIOValue::U8(value) => {
          let mut bytes = value.to_be_bytes().to_vec();
          accumulator.append(&mut bytes);
        }
        _ => (),
      }

      return accumulator;
    });

    let actual_vin = String::from_utf8(combined_vin).unwrap();

    return Some(actual_vin);
  }
}