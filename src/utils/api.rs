use chrono::{DateTime, Utc};
use log::{info, warn};
use uuid::Uuid;
use vehicle_management_service::apis::{
    public_trucks_api::ListPublicTrucksParams,
    trucks_api::{
        delete_truck_driver_card, list_truck_driver_cards, DeleteTruckDriverCardParams,
        ListTruckDriverCardsParams,
    },
};

use super::get_vehicle_management_api_config;

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
            warn!(
                "Failed to get truck ID by VIN [{}]: {}",
                vin.clone().unwrap(),
                err
            );
            return None;
        }
    }
}

/// Gets truck driver card.
///
/// API returns a list, but in reality there should always be just one.
///
/// # Arguments
/// * `truck_id` - Truck ID
///
/// # Returns
/// * `Option<String>` - Driver card ID
pub async fn get_truck_driver_card_id(truck_id: String) -> Option<String> {
    let Ok(driver_cards) = list_truck_driver_cards(
        &get_vehicle_management_api_config(),
        ListTruckDriverCardsParams {
            truck_id: truck_id.clone(),
        },
    )
    .await
    else {
        info!("Failed to get driver cards for truck [{}]", truck_id);
        return None;
    };
    assert!(
        driver_cards.len() <= 1,
        "Truck has more than one driver card"
    );
    let Some(driver_card) = driver_cards.first() else {
        info!("Truck [{}] has no driver card", truck_id);
        return None;
    };

    Some(driver_card.id.clone())
}

/// Deletes truck driver card from truck
///
/// # Arguments
/// * `truck_id` - Truck ID
/// * `driver_card_id` - Driver card ID
/// * `removed_at` - Time when the driver card was removed
pub async fn delete_truck_driver_card_by_id(
    truck_id: String,
    driver_card_id: String,
    removed_at: DateTime<Utc>,
) {
    match delete_truck_driver_card(
        &get_vehicle_management_api_config(),
        DeleteTruckDriverCardParams {
            truck_id: truck_id.clone(),
            driver_card_id: driver_card_id.clone(),
            x_driver_card_removed_at: removed_at.to_string(),
        },
    )
    .await
    {
        Ok(_) => info!(
            "Driver card [{}] deleted from truck [{}]",
            driver_card_id, truck_id
        ),
        Err(err) => warn!(
            "Failed to delete driver card [{}] from truck [{}]: {}",
            driver_card_id, truck_id, err
        ),
    };
}
