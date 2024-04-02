use serde::{Deserialize, Serialize};

use super::{Cacheable, CacheableU16};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheableTruckSpeed {
    speed: f32,
    timestamp: i64,
}

impl CacheableU16 for CacheableTruckSpeed {
  fn from_teltonika_event(speed: u16, timestamp: i64) -> Self {
    CacheableTruckSpeed {
      speed: speed as f32,
      timestamp: timestamp,
    }
  }
}

impl Cacheable for CacheableTruckSpeed {
  const FILE_PATH: &'static str = "truck_speed.json";
}