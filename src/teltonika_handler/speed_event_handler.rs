use std::path::Path;

use log::debug;
use nom_teltonika::AVLEventIO;

use super::TeltonikaEventHandler;
use crate::telematics_cache::{cacheable_truck_speed::CacheableTruckSpeed, Cacheable};

pub const SPEED_EVENT_HANDLER: TeltonikaEventHandler = TeltonikaEventHandler {
  event_id: 191,
  handler: handle_speed_event,
  purge: purge_speed_events_cache,
};

fn handle_speed_event(event: AVLEventIO, timestamp: i64, truck_id: Option<String>, base_cache_path: Box<Path>) {
  if let Some(truck_id) =  truck_id {
    debug!("Handling speed event for truck: {}", truck_id);
  } else {
    debug!("Caching speed event for yet unknown truck");
    let cache_result = CacheableTruckSpeed::from_teltonika_event(&event.value, timestamp)
      .expect("Error parsing speed event")
      .write_to_file(base_cache_path.to_owned().to_str().unwrap());
    if let Err(e) = cache_result {
      panic!("Error caching speed event: {:?}", e);
    }
  };
}

fn purge_speed_events_cache(_truck_id: String, base_cache_path: Box<Path>) {
  let cache = CacheableTruckSpeed::read_from_file(base_cache_path.to_str().unwrap());

  for _cached_truck_speed in cache.iter() {
    // TODO: Send these to backend in next PR
  }

  CacheableTruckSpeed::clear_cache(base_cache_path.to_owned().to_str().unwrap());
}