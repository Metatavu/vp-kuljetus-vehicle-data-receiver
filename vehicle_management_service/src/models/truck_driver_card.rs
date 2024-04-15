/*
 * Vehicle Management Services (vehicle-data-receiver)
 *
 * Vehicle Management Services (vehicle-data-receiver)
 *
 * The version of the OpenAPI document: 1.0.0
 * 
 * Generated by: https://openapi-generator.tech
 */

use crate::models;

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct TruckDriverCard {
    /// Driver card ID
    #[serde(rename = "id")]
    pub id: String,
}

impl TruckDriverCard {
    pub fn new(id: String) -> TruckDriverCard {
        TruckDriverCard {
            id,
        }
    }
}

