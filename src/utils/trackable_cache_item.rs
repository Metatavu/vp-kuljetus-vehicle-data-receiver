use chrono::Utc;
use vehicle_management_service::models::Trackable;

pub struct TrackableCacheItem {
    pub trackable: Trackable,
    pub updated_at: chrono::DateTime<Utc>,
}

impl TrackableCacheItem {
    pub fn new(trackable: Trackable) -> Self {
        Self {
            trackable,
            updated_at: chrono::Utc::now(),
        }
    }
}
