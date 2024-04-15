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
pub struct TelematicData {
    #[serde(rename = "timestamp")]
    pub timestamp: i64,
    #[serde(rename = "imei")]
    pub imei: String,
    #[serde(rename = "speed")]
    pub speed: f32,
    #[serde(rename = "latitude")]
    pub latitude: f64,
    #[serde(rename = "longitude")]
    pub longitude: f64,
}

impl TelematicData {
    pub fn new(timestamp: i64, imei: String, speed: f32, latitude: f64, longitude: f64) -> TelematicData {
        TelematicData {
            timestamp,
            imei,
            speed,
            latitude,
            longitude,
        }
    }
}

