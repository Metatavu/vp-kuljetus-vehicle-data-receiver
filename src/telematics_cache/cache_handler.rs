use std::path::PathBuf;

use log::debug;
use vehicle_management_service::{
    apis::trucks_api::CreateTruckLocationParams, models::TruckLocation,
};

use crate::{
    telematics_cache::Cacheable, teltonika::events::TeltonikaEventHandlers,
    utils::get_vehicle_management_api_config,
};

pub const PURGE_CHUNK_SIZE_ENV_KEY: &str = "PURGE_CHUNK_SIZE";
pub const DEFAULT_PURGE_CHUNK_SIZE: usize = 10;

pub struct CacheHandler {
    log_target: String,
    truck_id: String,
    base_cache_path: PathBuf,
}

impl CacheHandler {
    pub fn new(log_target: String, truck_id: String, base_cache_path: PathBuf) -> Self {
        Self {
            log_target,
            truck_id,
            base_cache_path,
        }
    }

    /// Purges the cached telematics data for a truck.
    pub async fn purge_cache(&self, purge_cache_size: usize) {
        self.purge_location_cache(purge_cache_size).await;

        for handler in TeltonikaEventHandlers::event_handlers(&self.log_target).iter() {
            handler
                .purge_cache(
                    self.truck_id.clone(),
                    self.base_cache_path.clone(),
                    purge_cache_size,
                )
                .await;
        }
    }

    /// Purges the cached locations data.
    async fn purge_location_cache(&self, purge_cache_size: usize) {
        let base_cache_path = self.base_cache_path.to_str().unwrap();
        let (cache, cache_size) = TruckLocation::read_from_file(base_cache_path, purge_cache_size);
        let mut failed_locations = Vec::new();

        let purge_cache_size = cache.len();
        debug!(target: &self.log_target,
            "Purging location cache of {purge_cache_size}/{cache_size} locations.",
        );

        for cached_location in cache.iter() {
            let result = vehicle_management_service::apis::trucks_api::create_truck_location(
                &get_vehicle_management_api_config(),
                CreateTruckLocationParams {
                    truck_id: self.truck_id.clone(),
                    truck_location: cached_location.clone(),
                },
            )
            .await;
            if let Err(err) = result {
                debug!(target: &self.log_target,
                    "Error sending location: {err:?}. Caching it for further use.",
                );
                failed_locations.push(cached_location.clone());
            }
        }
        let successful_locations_count = cache.len() - failed_locations.len();
        let failed_locations_count = failed_locations.len();
        debug!(target: &self.log_target,
            "Purged location cache of {successful_locations_count} locations. {failed_locations_count} failed to send.",
        );
        TruckLocation::clear_cache(base_cache_path);
        for failed_location in failed_locations.iter() {
            failed_location
                .write_to_file(base_cache_path)
                .expect("Error caching location");
        }
    }
}
