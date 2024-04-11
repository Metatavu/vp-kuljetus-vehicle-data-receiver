pub mod speed_event_handler;
pub mod teltonika_records_handler;
mod teltonika_event_handlers;
mod teltonika_vin_handler;

use nom_teltonika::AVLEventIOValue;

/// Converts an [AVLEventIOValue] to a big-endian byte vector.
fn avl_event_io_value_to_be_bytes(value: &AVLEventIOValue) -> Vec<u8> {
  match value {
    AVLEventIOValue::U64(value) => value.to_be_bytes().to_vec(),
    AVLEventIOValue::U32(value) => value.to_be_bytes().to_vec(),
    AVLEventIOValue::U16(value) => value.to_be_bytes().to_vec(),
    AVLEventIOValue::U8(value) => value.to_be_bytes().to_vec(),
    _ => Vec::new(),
  }
}

/// Converts an [AVLEventIOValue] to a u64.
fn avl_event_io_value_to_u64(value: &AVLEventIOValue) -> u64 {
  match value {
    AVLEventIOValue::U64(value) => *value,
    AVLEventIOValue::U32(value) => *value as u64,
    AVLEventIOValue::U16(value) => *value as u64,
    AVLEventIOValue::U8(value) => *value as u64,
    _ => 0,
  }
}