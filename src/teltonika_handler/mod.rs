use nom_teltonika::{AVLEventIOValue, AVLRecord};

const TELTONIKA_VIN_EVENT_IDS: [u16; 3] = [233, 234, 235];

pub struct TeltonikaRecordsHandler {}

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
}