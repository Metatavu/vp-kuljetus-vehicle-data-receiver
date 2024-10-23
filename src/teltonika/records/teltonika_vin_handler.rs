use nom_teltonika::{AVLEventIOValue, AVLRecord};

use crate::teltonika::avl_event_io_value_to_be_bytes;

const TELTONIKA_VIN_EVENT_IDS: [u16; 3] = [233, 234, 235];

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
pub fn get_truck_vin_from_records(teltonika_records: &Vec<AVLRecord>) -> Option<String> {
    let mut part_1 = None;
    let mut part_2 = None;
    let mut part_3 = None;
    for record in teltonika_records.iter() {
        for event in record.io_events.iter() {
            let is_vin_event = TELTONIKA_VIN_EVENT_IDS.contains(&event.id);
            if is_vin_event {
                match &event.id {
                    233 => part_1 = Some(event.value.clone()),
                    234 => part_2 = Some(event.value.clone()),
                    235 => part_3 = Some(event.value.clone()),
                    _ => (),
                }
            }
        }
        let is_complete = part_1.is_some() && part_2.is_some() && part_3.is_some();
        // If we have all three parts, we can break the loop
        if is_complete {
            break;
        }
    }

    return parse_vin(part_1, part_2, part_3);
}

/// Combines the three binary parts of the VIN into the full string representation.
///
/// # Arguments
/// * `part_1` - The first part of the VIN.
/// * `part_2` - The second part of the VIN.
/// * `part_3` - The third part of the VIN.
///
/// # Returns
/// * The full VIN if all three parts are present, otherwise None.
fn parse_vin(
    part_1: Option<AVLEventIOValue>,
    part_2: Option<AVLEventIOValue>,
    part_3: Option<AVLEventIOValue>,
) -> Option<String> {
    if part_1.is_some() && part_2.is_some() && part_3.is_some() {
        let part_1 = avl_event_io_value_to_be_bytes(&part_1.unwrap());
        let part_2 = avl_event_io_value_to_be_bytes(&part_2.unwrap());
        let part_3 = avl_event_io_value_to_be_bytes(&part_3.unwrap());
        let mut vin = Vec::new();
        vin.extend_from_slice(&part_1);
        vin.extend_from_slice(&part_2);
        vin.extend_from_slice(&part_3);

        return Some(String::from_utf8(vin).unwrap());
    }

    return None;
}
