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
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct TruckDriverCard {
    /// Driver card ID
    #[serde(rename = "id")]
    pub id: String,
    /// Timestamp for driver card insertion. Unix timestamp in milliseconds.
    #[serde(rename = "timestamp")]
    pub timestamp: i64,
}

impl TruckDriverCard {
    pub fn new(id: String, timestamp: i64) -> TruckDriverCard {
        TruckDriverCard {
            id,
            timestamp,
        }
    }
}

