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

/// Truck : Represent single truck
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Truck {
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<uuid::Uuid>,
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "plateNumber")]
    pub plate_number: String,
    #[serde(rename = "type")]
    pub r#type: Type,
    /// Truck identification number. This is unique for each truck and should be used as a hardware identifier for this specific truck. 
    #[serde(rename = "vin")]
    pub vin: String,
    /// Active vehicle id. This is the current vehicle that the truck is part of. It updates whenever the vehicle structure is updated. 
    #[serde(rename = "activeVehicleId", skip_serializing_if = "Option::is_none")]
    pub active_vehicle_id: Option<uuid::Uuid>,
    #[serde(rename = "creatorId", skip_serializing_if = "Option::is_none")]
    pub creator_id: Option<uuid::Uuid>,
    #[serde(rename = "createdAt", skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(rename = "lastModifierId", skip_serializing_if = "Option::is_none")]
    pub last_modifier_id: Option<uuid::Uuid>,
    #[serde(rename = "modifiedAt", skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<String>,
    /// Setting the archivedAt time marks the truck as archived. Trucks marked as archived will not appear in list requests unless archived filter is set to true. Archived truck cannot be updated, unless archivedAt is first set to null. 
    #[serde(rename = "archivedAt", skip_serializing_if = "Option::is_none")]
    pub archived_at: Option<String>,
}

impl Truck {
    /// Represent single truck
    pub fn new(plate_number: String, r#type: Type, vin: String) -> Truck {
        Truck {
            id: None,
            name: None,
            plate_number,
            r#type,
            vin,
            active_vehicle_id: None,
            creator_id: None,
            created_at: None,
            last_modifier_id: None,
            modified_at: None,
            archived_at: None,
        }
    }
}
/// 
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Type {
    #[serde(rename = "TRUCK")]
    Truck,
    #[serde(rename = "SEMI_TRUCK")]
    SemiTruck,
}

impl Default for Type {
    fn default() -> Type {
        Self::Truck
    }
}

