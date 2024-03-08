use serde::{Serialize, Deserialize};
///Represent single vehicle
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Vehicle {
    ///Setting the archivedAt time marks the vehicle as archived. Vehicles marked as archived will not appear in list requests unless archived filter is set to true. Archived vehicle cannot be updated, unless archivedAt is first set to null.
    #[serde(rename = "archivedAt")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archived_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(rename = "createdAt")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(rename = "creatorId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub creator_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "lastModifierId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_modifier_id: Option<String>,
    #[serde(rename = "modifiedAt")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<chrono::DateTime<chrono::Utc>>,
    ///List of attached towables to vehicle. Order of towables should reflect the order of towables in the vehicle where first towable is the closest to the truck.
    #[serde(rename = "towableIds")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub towable_ids: Vec<String>,
    ///Truck that towing the vehicle
    #[serde(rename = "truckId")]
    pub truck_id: String,
}
impl std::fmt::Display for Vehicle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}