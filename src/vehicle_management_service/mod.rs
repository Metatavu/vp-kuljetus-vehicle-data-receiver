use http::Response;
use httpclient::InMemoryBody;
use vehicle_management_service_client::{
  request::CreateTruckLocationRequired, Error, VehicleManagementServiceClientAuth as VehicleManagementServiceAuth, VehicleManagementServiceClientClient as VehicleManagementServiceClient
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
      .vin(vin.clone().unwrap().as_str())
      .await
      .expect("Failed to list public trucks");

    for truck in trucks {
      if truck.vin == vin.clone().unwrap() {
        return truck.id;
      }
    }

    return None;
  }

  /// Sends truck speed event
  ///
  /// # Arguments
  /// * `truck_id` - ID of the truck
  /// * `timestamp` - Timestamp of the event
  /// * `speed` - Speed of the truck
  ///
  /// # Returns
  /// * `Result<(), Error<Response<InMemoryBody>>>` - Result of the operation
  pub async fn send_truck_speed(truck_id: String, timestamp: i64, speed: f64) -> Result<(), Error<Response<InMemoryBody>>> {
    Self::get_vehicle_management_service_client()
      .create_truck_speed(speed, timestamp, &truck_id).await
  }

  /// Gets authenticated Vehicle Management Service client
  fn get_vehicle_management_service_client() -> VehicleManagementServiceClient {
      let vehicle_management_service_api_key = read_string_env_variable("VEHICLE_MANAGEMENT_SERVICE_API_KEY");

      return VehicleManagementServiceClient::with_auth(
          VehicleManagementServiceAuth::ApiKeyAuth { x_api_key: vehicle_management_service_api_key }
      );
  }

  /// Sends truck location event
  ///
  /// # Arguments
  /// * `truck_id` - ID of the truck
  /// * `timestamp` - Timestamp of the event
  /// * `latitude` - Latitude of the truck
  /// * `longitude` - Longitude of the truck
  /// * `heading` - Heading of the truck
  ///
  /// # Returns
  /// * `Result<(), Error<Response<InMemoryBody>>>` - Result of the operation
  pub async fn send_truck_location(truck_id: String, timestamp: i64, latitude: f64, longitude: f64, heading: f64) -> Result<(), Error<Response<InMemoryBody>> > {
    Self::get_vehicle_management_service_client()
      .create_truck_location(
        CreateTruckLocationRequired {
          latitude,
          longitude,
          heading,
          timestamp,
          truck_id: &truck_id
        })
        .await
  }
}