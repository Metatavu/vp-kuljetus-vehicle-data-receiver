use std::path::PathBuf;

use log::debug;
use vehicle_management_service::{apis::trucks_api::CreateTruckLocationParams, models::TruckLocation};

use crate::{
    telematics_cache::Cacheable, teltonika::events::TeltonikaEventHandlers, utils::get_vehicle_management_api_config,
};

/// Environment variable key for the cache purge size.
pub const PURGE_CHUNK_SIZE_ENV_KEY: &str = "PURGE_CHUNK_SIZE";

/// Default cache purge size. If the environment variable is not set, this value will be used.
///
/// The purge chunk size is the amount of cached data that will be processed from the cache files at once.
pub const DEFAULT_PURGE_CHUNK_SIZE: usize = 0;

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
    ///
    /// # Arguments
    /// * `purge_cache_size` - The amount of cached data that will be processed from the cache files at once.
    pub async fn purge_cache(&self, purge_cache_size: usize) {
        self.purge_location_cache(purge_cache_size).await;

        for handler in TeltonikaEventHandlers::event_handlers(&self.log_target).iter() {
            handler
                .purge_cache(self.truck_id.clone(), self.base_cache_path.clone(), purge_cache_size)
                .await;
        }
    }

    /// Purges the cached locations data.
    ///
    /// # Arguments
    /// * `purge_cache_size` - The amount of cached data that will be processed from the cache files at once.
    async fn purge_location_cache(&self, purge_cache_size: usize) {
        let (cache, cache_size) = TruckLocation::take_from_file(self.base_cache_path.clone(), purge_cache_size);
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
        TruckLocation::write_vec_to_file(failed_locations, self.base_cache_path.clone())
            .expect("Error caching locations");
    }
}

#[cfg(test)]
mod tests {
    use rand::{thread_rng, RngCore};
    use uuid::Uuid;
    use vehicle_management_service::models::{
        TemperatureReading, TemperatureReadingSourceType, TruckDriveState, TruckDriveStateEnum, TruckDriverCard,
        TruckLocation, TruckOdometerReading, TruckSpeed,
    };

    use crate::{
        telematics_cache::{cache_handler::CacheHandler, Cacheable},
        utils::{
            imei::get_random_imei_of_length,
            test_utils::{get_temp_dir_path, mock_server, MockServerExt},
        },
    };

    #[tokio::test]
    async fn test_purge_cache() {
        let _mocks = mock_server().start_all_mocks();
        let base_cache_path = get_temp_dir_path();
        let imei = get_random_imei_of_length(15);
        let truck_id = Uuid::new_v4().to_string();
        let cache_handler = CacheHandler::new(imei.clone(), truck_id, base_cache_path.clone());

        let mut locations = Vec::new();
        let mut truck_speeds = Vec::new();
        let mut truck_driver_cards = Vec::new();
        let mut truck_drive_states = Vec::new();
        let mut truck_odometer_readings = Vec::new();
        let mut truck_temperature_readings = Vec::new();

        let hardware_sensor_ids = (0..4).map(|_| thread_rng().next_u64()).collect::<Vec<u64>>();
        println!("{:?}", hardware_sensor_ids);
        for i in 0..10 {
            let date_time = chrono::Utc::now() + chrono::Duration::minutes(i);
            let timestamp = date_time.timestamp();
            locations.push(TruckLocation::new(
                timestamp,
                0.0 + i as f64,
                0.0 + i as f64,
                0.0 + i as f64,
            ));
            truck_speeds.push(TruckSpeed::new(timestamp, 0.0 + i as f32));
            truck_driver_cards.push(TruckDriverCard {
                id: String::from("1099483935000001"),
                timestamp,
                removed_at: None,
            });
            truck_drive_states.push(TruckDriveState {
                id: None,
                timestamp,
                state: TruckDriveStateEnum::Drive,
                driver_id: None,
                driver_card_id: None,
            });
            truck_odometer_readings.push(TruckOdometerReading::new(timestamp, i as i32));

            let mut curr_temperature_readings = Vec::new();
            for hardware_sensor_id in &hardware_sensor_ids {
                curr_temperature_readings.push(TemperatureReading::new(
                    imei.clone(),
                    hardware_sensor_id.to_string(),
                    i as f32,
                    timestamp,
                    TemperatureReadingSourceType::Truck,
                ));
            }
            truck_temperature_readings.push(curr_temperature_readings);
        }

        TruckLocation::write_vec_to_file(locations, base_cache_path.clone()).unwrap();
        TruckSpeed::write_vec_to_file(truck_speeds, base_cache_path.clone()).unwrap();
        TruckDriverCard::write_vec_to_file(truck_driver_cards, base_cache_path.clone()).unwrap();
        TruckDriveState::write_vec_to_file(truck_drive_states, base_cache_path.clone()).unwrap();
        TruckOdometerReading::write_vec_to_file(truck_odometer_readings, base_cache_path.clone()).unwrap();
        Vec::<TemperatureReading>::write_vec_to_file(truck_temperature_readings, base_cache_path.clone()).unwrap();

        let (locations_cache, _) = TruckLocation::read_from_file(base_cache_path.clone(), 0);
        let (truck_speeds_cache, _) = TruckSpeed::read_from_file(base_cache_path.clone(), 0);
        let (truck_driver_cards_cache, _) = TruckDriverCard::read_from_file(base_cache_path.clone(), 0);
        let (truck_drive_states_cache, _) = TruckDriveState::read_from_file(base_cache_path.clone(), 0);
        let (truck_odometer_readings_cache, _) = TruckOdometerReading::read_from_file(base_cache_path.clone(), 0);
        let (truck_temperature_readings_cache, _) =
            Vec::<TemperatureReading>::read_from_file(base_cache_path.clone(), 0);

        assert_eq!(locations_cache.len(), 10);
        assert_eq!(truck_speeds_cache.len(), 10);
        assert_eq!(truck_driver_cards_cache.len(), 10);
        assert_eq!(truck_drive_states_cache.len(), 10);
        assert_eq!(truck_odometer_readings_cache.len(), 10);
        assert_eq!(truck_temperature_readings_cache.len(), 10);

        cache_handler.purge_cache(5).await;

        let (locations_cache, _) = TruckLocation::read_from_file(base_cache_path.clone(), 0);
        let (truck_speeds_cache, _) = TruckSpeed::read_from_file(base_cache_path.clone(), 0);
        let (truck_driver_cards_cache, _) = TruckDriverCard::read_from_file(base_cache_path.clone(), 0);
        let (truck_drive_states_cache, _) = TruckDriveState::read_from_file(base_cache_path.clone(), 0);
        let (truck_odometer_readings_cache, _) = TruckOdometerReading::read_from_file(base_cache_path.clone(), 0);
        let (truck_temperature_readings_cache, _) =
            Vec::<TemperatureReading>::read_from_file(base_cache_path.clone(), 0);

        assert_eq!(locations_cache.len(), 5);
        assert_eq!(truck_speeds_cache.len(), 5);
        assert_eq!(truck_driver_cards_cache.len(), 5);
        assert_eq!(truck_drive_states_cache.len(), 5);
        assert_eq!(truck_odometer_readings_cache.len(), 5);
        assert_eq!(truck_temperature_readings_cache.len(), 5);
    }
}
