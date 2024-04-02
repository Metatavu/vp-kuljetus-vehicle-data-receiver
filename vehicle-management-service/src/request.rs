pub mod update_driver_card;
pub mod list_public_trucks;
pub mod receive_telematic_data;
pub use update_driver_card::UpdateDriverCardRequest;
pub use list_public_trucks::ListPublicTrucksRequest;
pub use receive_telematic_data::{
    ReceiveTelematicDataRequest, ReceiveTelematicDataRequired,
};