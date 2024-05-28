/*
 * VP-Kuljetus Vehicle Management Services
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
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

