pub mod create_truck_driver_card;
pub mod delete_truck_driver_card;
pub mod create_truck_speed;
pub mod receive_telematic_data;
pub use create_truck_driver_card::CreateTruckDriverCardRequest;
pub use delete_truck_driver_card::DeleteTruckDriverCardRequest;
pub use create_truck_speed::CreateTruckSpeedRequest;
pub use receive_telematic_data::{
    ReceiveTelematicDataRequest, ReceiveTelematicDataRequired,
};