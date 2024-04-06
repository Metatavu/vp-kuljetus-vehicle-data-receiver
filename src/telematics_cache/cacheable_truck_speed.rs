use nom_teltonika::AVLEventIOValue;
use serde::{Deserialize, Serialize};

use super::Cacheable;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheableTruckSpeed {
  pub speed: f32,
  pub timestamp: i64,
}

impl Cacheable for CacheableTruckSpeed {
  const FILE_PATH: &'static str = "truck_speed_cache.json";

  fn from_teltonika_event(value: &AVLEventIOValue, timestamp: i64) -> Option<Self> {
    let speed = match value {
      AVLEventIOValue::U16(val) => Some(val.clone() as f32),
      _ => None
    };
    if speed.is_none() {
      return None
    };
    Some(CacheableTruckSpeed { speed: speed.unwrap(), timestamp: timestamp })
  }
}