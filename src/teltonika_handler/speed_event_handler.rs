use http::Response;
use httpclient::InMemoryBody;
use vehicle_management_service_client::Error;
use vehicle_management_service_client::request::CreateTruckSpeedRequest;
use super::{avl_event_io_value_to_u64, teltonika_event_handlers::TeltonikaEventHandler};
use crate::telematics_cache::Cacheable;
use crate::vehicle_management_service::VehicleManagementService;

pub struct SpeedEventHandler {}

impl TeltonikaEventHandler<CreateTruckSpeedRequest> for SpeedEventHandler {
  fn get_event_id(&self) -> u16 {
    191
  }

  async fn send_event(&self, event_data: CreateTruckSpeedRequest, truck_id: String) -> Result<(), Error<Response<InMemoryBody>>> {
    VehicleManagementService::send_truck_speed(truck_id, event_data.timestamp, event_data.speed).await
  }

  fn process_event_data(&self, event: &nom_teltonika::AVLEventIOValue, truck_id: String, timestamp: i64) -> CreateTruckSpeedRequest {
      CreateTruckSpeedRequest {
          id: None,
          speed: avl_event_io_value_to_u64(event) as f64,
          timestamp,
          truck_id,

      }
  }
}

impl Cacheable for CreateTruckSpeedRequest {

  const FILE_PATH: &'static str = "truck_speed_cache.json";

  fn from_teltonika_event(value: &nom_teltonika::AVLEventIOValue, timestamp: i64) -> Option<Self> {
    let speed = match value {
      nom_teltonika::AVLEventIOValue::U16(val) => Some(val.clone() as f64),
      _ => None
    };
    if speed.is_none() {
      return None
    };
    Some(CreateTruckSpeedRequest { id: None, speed: speed.unwrap(), timestamp, truck_id: "".to_string() })
  }
}