use vehicle_management_service_client::{
  VehicleManagementServiceClientAuth as VehicleManagementServiceAuth,
  VehicleManagementServiceClientClient as VehicleManagementServiceClient
};

use crate::read_string_env_variable;

pub struct VehicleManagementService {}

impl VehicleManagementService {

  /// Gets truck ID by VIN
  ///
  /// # Arguments
  /// * `vin` - VIN of the truck
  ///
  /// # Returns
  /// * `Option<String>` - Truck ID
  pub async fn get_truck_id_by_vin(vin: &Option<String>) -> Option<String> {
    if vin.is_none() {
      return None;
    }

    let trucks = Self::get_vehicle_management_service_client()
      .list_public_trucks()
      .await
      .expect("Failed to list public trucks");

    for truck in trucks {
      if truck.vin == vin.clone().unwrap() {
        return truck.id;
      }
    }

    return None;
  }

  /// Gets authenticated Vehicle Management Service client
  fn get_vehicle_management_service_client() -> VehicleManagementServiceClient {
      let vehicle_management_service_api_key = read_string_env_variable("VEHICLE_MANAGEMENT_SERVICE_API_KEY");

      return VehicleManagementServiceClient::with_auth(
          VehicleManagementServiceAuth::ApiKeyAuth { x_api_key: vehicle_management_service_api_key }
      );
  }
}