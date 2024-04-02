use log::debug;
use nom_teltonika::{AVLEventIO, AVLEventIOValue};

use crate::telematics_cache::{cacheable_truck_speed::CacheableTruckSpeed, CacheableU16, Cacheable};

use super::TeltonikaEventHandler;

pub const SPEED_EVENT_HANDLER: TeltonikaEventHandler = TeltonikaEventHandler {
  event_id: 191,
  handler: handle_speed_event,
};

fn handle_speed_event(event: AVLEventIO, timestamp: i64, truck_id: Option<String>) {
  if let Some(truck_id) =  truck_id {
    debug!("Handling speed event for truck: {}", truck_id);

  } else {
    debug!("Caching speed event for yet unknown truck");
    let value = match event.value {
      AVLEventIOValue::U16(value) => value,
      _ => {
        debug!("Unexpected value type for speed event: {:?}", event.value);
        return;
      }
    };
    let cache_result = CacheableTruckSpeed::from_teltonika_event(value, timestamp)
      .write_to_file("".to_string());
    if let Err(e) = cache_result {
      debug!("Error caching speed event: {:?}", e);
    }
  };
  println!("Handling speed event: {:?}", event);
}