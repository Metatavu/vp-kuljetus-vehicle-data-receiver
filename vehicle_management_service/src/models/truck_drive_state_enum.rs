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

/// 
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum TruckDriveStateEnum {
    #[serde(rename = "REST")]
    Rest,
    #[serde(rename = "DRIVER_AVAILABLE")]
    DriverAvailable,
    #[serde(rename = "WORK")]
    Work,
    #[serde(rename = "DRIVE")]
    Drive,
    #[serde(rename = "ERROR")]
    Error,
    #[serde(rename = "NOT_AVAILABLE")]
    NotAvailable,

}

impl ToString for TruckDriveStateEnum {
    fn to_string(&self) -> String {
        match self {
            Self::Rest => String::from("REST"),
            Self::DriverAvailable => String::from("DRIVER_AVAILABLE"),
            Self::Work => String::from("WORK"),
            Self::Drive => String::from("DRIVE"),
            Self::Error => String::from("ERROR"),
            Self::NotAvailable => String::from("NOT_AVAILABLE"),
        }
    }
}

impl Default for TruckDriveStateEnum {
    fn default() -> TruckDriveStateEnum {
        Self::Rest
    }
}

