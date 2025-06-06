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

/// Thermometer : Represents a thermometer attached to a truck or towable
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Thermometer {
    /// Unique identifier for the thermometer
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<uuid::Uuid>,
    /// Name of the thermometer
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// MAC address of the thermometer. It is unique and stays with the device.
    #[serde(rename = "macAddress")]
    pub mac_address: String,
    /// The ID of the entity currently associated with the thermometer.
    #[serde(rename = "entityId")]
    pub entity_id: uuid::Uuid,
    /// The type of the entity to which the thermometer is attached (e.g., \"towable\", \"truck\", etc.)
    #[serde(rename = "entityType")]
    pub entity_type: EntityType,
    #[serde(rename = "creatorId", skip_serializing_if = "Option::is_none")]
    pub creator_id: Option<uuid::Uuid>,
    #[serde(rename = "createdAt", skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(rename = "lastModifierId", skip_serializing_if = "Option::is_none")]
    pub last_modifier_id: Option<uuid::Uuid>,
    #[serde(rename = "modifiedAt", skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<String>,
    /// Setting the archivedAt time marks the thermometer as archived. Thermometers marked as archived will not appear in list requests unless includeArchived filter is set to true. Archived thermometer cannot be updated, unless archivedAt is first set to null. 
    #[serde(rename = "archivedAt", skip_serializing_if = "Option::is_none")]
    pub archived_at: Option<String>,
}

impl Thermometer {
    /// Represents a thermometer attached to a truck or towable
    pub fn new(mac_address: String, entity_id: uuid::Uuid, entity_type: EntityType) -> Thermometer {
        Thermometer {
            id: None,
            name: None,
            mac_address,
            entity_id,
            entity_type,
            creator_id: None,
            created_at: None,
            last_modifier_id: None,
            modified_at: None,
            archived_at: None,
        }
    }
}
/// The type of the entity to which the thermometer is attached (e.g., \"towable\", \"truck\", etc.)
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum EntityType {
    #[serde(rename = "truck")]
    Truck,
    #[serde(rename = "towable")]
    Towable,
}

impl Default for EntityType {
    fn default() -> EntityType {
        Self::Truck
    }
}

