use log::{info, warn};
use uuid::Uuid;
use vehicle_management_service::{
    apis::{
        public_trucks_api::ListPublicTrucksParams,
        trackables_api::GetTrackableByImeiParams,
        trucks_api::{list_truck_driver_cards, ListTruckDriverCardsParams},
    },
    models::{Trackable, TruckDriverCard},
};

use super::get_vehicle_management_api_config;

pub async fn get_trackable(imei: &str) -> Option<Trackable> {
    match vehicle_management_service::apis::trackables_api::get_trackable_by_imei(
        &get_vehicle_management_api_config(),
        GetTrackableByImeiParams { imei: imei.to_string() },
    )
    .await
    {
        Ok(trackable) => Some(trackable),
        Err(err) => {
            warn!("Failed to get trackable by IMEI [{}]: {}", imei, err);
            None
        }
    }
}

/// Gets truck ID by VIN
///
/// This function will get the truck ID by the VIN.
///
/// # Arguments
/// * `vin` - VIN of the truck
///
/// # Returns
/// * `Option<Uuid>` - Truck ID
pub async fn get_truck_id_by_vin(vin: &Option<String>) -> Option<Uuid> {
    if vin.is_none() {
        return None;
    }

    match vehicle_management_service::apis::public_trucks_api::list_public_trucks(
        &get_vehicle_management_api_config(),
        ListPublicTrucksParams {
            vin: vin.clone(),
            first: None,
            max: None,
        },
    )
    .await
    {
        Ok(trucks) => {
            return trucks
                .iter()
                .find(|truck| truck.vin == vin.clone().unwrap())
                .map(|truck| truck.id.clone())
                .unwrap_or(None)
        }
        Err(err) => {
            warn!("Failed to get truck ID by VIN [{}]: {}", vin.clone().unwrap(), err);
            return None;
        }
    }
}

pub async fn fetch_all_driver_cards_in_truck(truck_id: &str) -> Option<Vec<TruckDriverCard>> {
    let Ok(driver_cards) = list_truck_driver_cards(
        &get_vehicle_management_api_config(),
        ListTruckDriverCardsParams {
            truck_id: truck_id.to_string(),
        },
    )
    .await
    else {
        info!("Failed to get driver cards for truck [{}]", truck_id);
        return None;
    };
    Some(driver_cards)
}
