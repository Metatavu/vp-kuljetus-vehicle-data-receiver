pub mod connection;
pub mod events;
pub mod records;
use log::warn;
use nom_teltonika::{AVLEventIO, AVLEventIOValue};
use serde::de::value;
use vehicle_management_service::models::{TruckDriveStateEnum, TruckDriverCard};

use crate::utils::date_time_from_timestamp;

/// The event ID for the event describing driver one card presence in tachograph.
const DRIVER_ONE_CARD_PRESENCE_EVENT_ID: u16 = 187;

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

/// Converts an [AVLEventIOValue] to a u32. Will panic if the value is not a bigger than u32.
fn avl_event_io_value_to_u32(value: &AVLEventIOValue) -> u32 {
    match value {
        AVLEventIOValue::U32(value) => *value,
        AVLEventIOValue::U16(value) => *value as u32,
        AVLEventIOValue::U8(value) => *value as u32,
        _ => panic!("Value is bigger than u32"),
    }
}

/// Converts an [AVLEventIOValue] to a u16. Will panic if the value is not a u16.
fn avl_event_io_value_to_u16(value: &AVLEventIOValue) -> u16 {
    match value {
        AVLEventIOValue::U16(value) => *value,
        AVLEventIOValue::U8(value) => *value as u16,
        AVLEventIOValue::U32(value) => *value as u16,
        AVLEventIOValue::U64(value) => *value as u16,
        _ => panic!("Value is not a u16"),
    }
}

/// Converts an [AVLEventIOValue] to a u8. Will panic if the value is not a u8.
fn avl_event_io_value_to_u8(value: &AVLEventIOValue) -> u8 {
    match value {
        AVLEventIOValue::U8(value) => *value,
        _ => panic!("Value is not a u8"),
    }
}

/// Converts a list of [AVLEventIO] to a [TruckDriverCard].
///
/// If either the MSB or LSB part of the driver card is 0, it is considered invalid and None is returned.
/// TODO: Investigate if in the case of valid driver card id the length of MSB and LSB fields are always same.
///
/// See [Teltonika Documentation](https://wiki.teltonika-gps.com/view/DriverID) for more detailed information.
fn driver_card_events_to_truck_driver_card(
    timestamp: i64,
    events: &Vec<&AVLEventIO>,
    imei: &str,
) -> Option<TruckDriverCard> {
    let card_present = events
        .iter()
        .find(|event| event.id == DRIVER_ONE_CARD_PRESENCE_EVENT_ID);

    let Some(driver_card_msb_part) = driver_card_part_from_event(events, 195) else {
        warn!(target: imei, "Driver card MSB part was 0");

        return None;
    };
    let Some(driver_card_lsb_part) = driver_card_part_from_event(events, 196) else {
        warn!(target: imei, "Driver card LSB part was 0");

        return None;
    };
    let id = format!("{}{}", driver_card_msb_part, driver_card_lsb_part);

    let removed_at = match card_present {
        Some(card_present) => get_card_removal_time_from_event(card_present, timestamp),
        None => None,
    };

    assert!(id.len() == 16);

    return Some(TruckDriverCard {
        id,
        timestamp,
        removed_at,
    });
}

/// Returns the time of driver card removal from an [AVLEventIO] event.
///
/// If the value of the event is 0 (card is not present), the records timestamp is returned.
///
/// # Arguments
/// - `event` - The driver card present event to.
/// - `timestamp` - The timestamp of the record.
///
/// # Returns
/// The time of driver card removal as a String in RFC3339 format or None if the card is present.
fn get_card_removal_time_from_event(event: &AVLEventIO, timestamp: i64) -> Option<String> {
    match avl_event_io_value_to_u8(&event.value) {
        0 => Some(date_time_from_timestamp(timestamp).to_rfc3339()),
        _ => None,
    }
}

/// Converts a Driver Card part [AVLEventIO] to a String.
///
/// See [Teltonika Documentation](https://wiki.teltonika-gps.com/view/DriverID) for more detailed information.
fn driver_card_part_event_to_string(event: &AVLEventIO) -> String {
    let driver_one_card_part = avl_event_io_value_to_u64(&event.value).to_be_bytes().to_vec();
    let Ok(part) = String::from_utf8(driver_one_card_part) else {
        panic!("Invalid driver one card part data");
    };

    return part;
}

/// Returns a driver card part as String from a list of [AVLEventIO].
///
/// If either the driver card part is 0, it is considered invalid and None is returned.
/// TODO: Investigate if in the case of valid driver card id the length of MSB and LSB fields are always same.
///
/// See [Teltonika Documentation](https://wiki.teltonika-gps.com/view/DriverID) for more detailed information.
fn driver_card_part_from_event(events: &Vec<&AVLEventIO>, event_id: u16) -> Option<String> {
    let driver_card_part = events
        .iter()
        .find(|event| event.id == event_id)
        .expect(&format!("Driver card part event not found {event_id}"));

    if driver_card_part.value == AVLEventIOValue::U64(0) {
        return None;
    }

    return Some(driver_card_part_event_to_string(driver_card_part));
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

#[cfg(test)]
mod tests {
    #[test]
    fn test_avl_event_io_value_to_u32() {
        use super::avl_event_io_value_to_u32;
        use nom_teltonika::AVLEventIOValue;

        assert_eq!(avl_event_io_value_to_u32(&AVLEventIOValue::U32(1)), 1);
        assert_eq!(avl_event_io_value_to_u32(&AVLEventIOValue::U16(1)), 1);
        assert_eq!(avl_event_io_value_to_u32(&AVLEventIOValue::U8(1)), 1);
    }

    #[test]
    #[should_panic]
    fn test_avl_event_io_value_to_u32_panic() {
        use super::avl_event_io_value_to_u32;
        use nom_teltonika::AVLEventIOValue;

        assert_eq!(avl_event_io_value_to_u32(&AVLEventIOValue::U64(1)), 1);
    }

    #[test]
    fn test_avl_event_io_value_to_u8() {
        use super::avl_event_io_value_to_u8;
        use nom_teltonika::AVLEventIOValue;

        assert_eq!(avl_event_io_value_to_u8(&AVLEventIOValue::U8(1)), 1);
    }

    #[test]
    #[should_panic]
    fn test_avl_event_io_value_to_u8_panic() {
        use super::avl_event_io_value_to_u8;
        use nom_teltonika::AVLEventIOValue;

        assert_eq!(avl_event_io_value_to_u8(&AVLEventIOValue::U64(1)), 1);
    }

    #[test]
    fn test_avl_event_io_value_to_u64() {
        use super::avl_event_io_value_to_u64;
        use nom_teltonika::AVLEventIOValue;

        assert_eq!(avl_event_io_value_to_u64(&AVLEventIOValue::U64(1)), 1);
        assert_eq!(avl_event_io_value_to_u64(&AVLEventIOValue::U32(1)), 1);
        assert_eq!(avl_event_io_value_to_u64(&AVLEventIOValue::U16(1)), 1);
        assert_eq!(avl_event_io_value_to_u64(&AVLEventIOValue::U8(1)), 1);
    }

    #[test]
    fn test_get_driver_card_removal_time_from_event() {
        use super::get_card_removal_time_from_event;
        use nom_teltonika::{AVLEventIO, AVLEventIOValue};

        let driver_one_card_not_present_event = AVLEventIO {
            id: 187,
            value: AVLEventIOValue::U8(0),
        };
        let driver_one_card_present_event = AVLEventIO {
            id: 187,
            value: AVLEventIOValue::U8(1),
        };
        let timestamp = 1616425200;

        assert!(get_card_removal_time_from_event(&driver_one_card_not_present_event, timestamp).is_some());
        assert!(get_card_removal_time_from_event(&driver_one_card_present_event, timestamp).is_none())
    }
}
