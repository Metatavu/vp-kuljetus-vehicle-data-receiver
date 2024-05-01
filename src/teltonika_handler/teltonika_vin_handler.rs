use nom_teltonika::AVLEventIOValue;

use super::avl_event_io_value_to_be_bytes;

/// Struct to hold the binary parts of a Teltonika VIN
pub struct TeltonikaVinHandler {
    part_1: Option<Vec<u8>>,
    part_2: Option<Vec<u8>>,
    part_3: Option<Vec<u8>>,
}

impl TeltonikaVinHandler {
    pub fn get_teltonika_vin_event_ids(&self) -> [u16; 3] {
        return [233, 234, 235];
    }

    pub fn new() -> Self {
        TeltonikaVinHandler {
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
