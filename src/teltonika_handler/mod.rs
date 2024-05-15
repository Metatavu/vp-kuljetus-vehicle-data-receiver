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
///
/// See [Teltonika Documentation](https://wiki.teltonika-gps.com/view/DriverID) for more detailed information.
fn driver_card_events_to_truck_driver_card(events: &Vec<&AVLEventIO>) -> TruckDriverCard {
    let driver_card_msb_part = driver_card_part_from_event(events, 195);
    let driver_card_lsb_part = driver_card_part_from_event(events, 196);
    let id = format!("{}{}", driver_card_msb_part, driver_card_lsb_part);

    assert!(id.len() == 16);

    return TruckDriverCard { id };
}
/// Converts a Driver Card part [AVLEventIO] to a String.
///
/// See [Teltonika Documentation](https://wiki.teltonika-gps.com/view/DriverID) for more detailed information.
fn driver_card_part_event_to_string(event: &AVLEventIO) -> String {
    let driver_one_card_msb = avl_event_io_value_to_u64(&event.value)
        .to_be_bytes()
        .to_vec();
    let Ok(test) = String::from_utf8(driver_one_card_msb) else {
        panic!("Invalid driver one card data");
    };
    let test = test.chars().rev().collect::<String>();

    return test;
}

/// Returns a driver card part as String from a list of [AVLEventIO].
///
/// See [Teltonika Documentation](https://wiki.teltonika-gps.com/view/DriverID) for more detailed information.
fn driver_card_part_from_event(events: &Vec<&AVLEventIO>, event_id: u16) -> String {
    let driver_card_part = events
        .iter()
        .find(|event| event.id == event_id)
        .expect(&format!("Driver card part event not found {event_id}"));

    return driver_card_part_event_to_string(driver_card_part);
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
