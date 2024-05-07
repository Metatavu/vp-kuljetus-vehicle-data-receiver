pub mod driver_one_card_id_event_handler;
pub mod driver_one_drive_state_event_handler;
pub mod speed_event_handler;
mod teltonika_event_handlers;
pub mod teltonika_records_handler;
mod teltonika_vin_handler;

use nom_teltonika::{AVLEventIO, AVLEventIOValue};
use vehicle_management_service::models::{TruckDriveStateEnum, TruckDriverCard};

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

/// Converts a list of [AVLEventIO] to a [TruckDriverCard].
fn driver_card_events_to_truck_driver_card(events: &Vec<&AVLEventIO>) -> TruckDriverCard {
    let driver_one_card_msb = events
        .iter()
        .find(|event| event.id == 195)
        .expect("Driver one card MSB event not found");
    let driver_one_card_lsb = events
        .iter()
        .find(|event| event.id == 196)
        .expect("Driver one card LSB event not found");
    let driver_one_card_msb = avl_event_io_value_to_u64(&driver_one_card_msb.value).to_be_bytes();
    let driver_one_card_lsb = avl_event_io_value_to_u64(&driver_one_card_lsb.value).to_be_bytes();
    let driver_one_card_part_1 = driver_one_card_msb
        .iter()
        .rev()
        .map(|byte| *byte as char)
        .collect::<String>();
    let driver_one_card_part_2 = driver_one_card_lsb
        .iter()
        .rev()
        .map(|byte| *byte as char)
        .collect::<String>();
    let id = driver_one_card_part_2 + &driver_one_card_part_1;

    assert!(id.len() == 16);

    return TruckDriverCard { id };
}

/// Trait for converting an [AVLEventIOValue] to a value used by Vehicle Management API.
trait FromAVLEventIoValue {
    fn from_avl_event_io_value(value: &AVLEventIOValue) -> Self;
}

/// Implementation of [FromAVLEventIoValue] for [TruckDriveStateEnum].
impl FromAVLEventIoValue for TruckDriveStateEnum {
    fn from_avl_event_io_value(value: &AVLEventIOValue) -> Self {
        match value {
            AVLEventIOValue::U8(value) => match value {
                0 => TruckDriveStateEnum::Rest,
                1 => TruckDriveStateEnum::DriverAvailable,
                2 => TruckDriveStateEnum::Work,
                3 => TruckDriveStateEnum::Drive,
                6 => TruckDriveStateEnum::Error,
                _ => TruckDriveStateEnum::NotAvailable,
            },
            _ => TruckDriveStateEnum::NotAvailable,
        }
    }
}
