use vehicle_management_service::{
    apis::{
        trucks_api::{CreateDriveStateError, CreateDriveStateParams},
        Error,
    },
    models::TruckDriveState,
};

use crate::{telematics_cache::Cacheable, utils::get_vehicle_management_api_config};

use super::teltonika_event_handlers::TeltonikaEventHandler;

pub struct DriverOneDriveStateEventHandler {}

impl TeltonikaEventHandler<TruckDriveState, Error<CreateDriveStateError>>
    for DriverOneDriveStateEventHandler
{
    fn get_event_ids(&self) -> Vec<u16> {
        vec![184]
    }

    fn send_event(
        event_data: &TruckDriveState,
        truck_id: String,
    ) -> Result<(), Error<CreateDriveStateError>> {
        vehicle_management_service::apis::trucks_api::create_drive_state(
            &get_vehicle_management_api_config(),
            CreateDriveStateParams {
                truck_id: truck_id.clone(),
                truck_drive_state: event_data.clone(),
            },
        )
    }

    fn process_event_data(
        events: &Vec<&nom_teltonika::AVLEventIO>,
        timestamp: i64,
    ) -> TruckDriveState {
        todo!()
        // return TruckDriveState {};
    }
}

impl Cacheable for TruckDriveState {
    const FILE_PATH: &'static str = "truck_drive_state_cache.json";

    fn from_teltonika_record(_: &nom_teltonika::AVLRecord) -> Option<Self> {
        None
    }
}
