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
pub struct TruckLocation {
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<uuid::Uuid>,
    /// Timestamp for truck speed
    #[serde(rename = "timestamp")]
    pub timestamp: i64,
    /// Latitude in degrees.
    #[serde(rename = "latitude")]
    pub latitude: f64,
    /// Longitude in degrees.
    #[serde(rename = "longitude")]
    pub longitude: f64,
    /// heading in degrees.
    #[serde(rename = "heading")]
    pub heading: f64,
}

impl TruckLocation {
    pub fn new(timestamp: i64, latitude: f64, longitude: f64, heading: f64) -> TruckLocation {
        TruckLocation {
            id: None,
            timestamp,
            latitude,
            longitude,
            heading,
        }
    }
}

