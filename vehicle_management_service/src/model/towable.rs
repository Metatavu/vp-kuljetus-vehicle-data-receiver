use serde::{Serialize, Deserialize};
///Represent single towable
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Towable {
    ///Setting the archivedAt time marks the towable as archived. Towables marked as archived will not appear in list requests unless archived filter is set to true. Archived towable cannot be updated, unless archivedAt is first set to null.
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "plateNumber")]
    pub plate_number: String,
    #[serde(rename = "type")]
    pub type_: String,
    ///Towable identification number. This is unique for each towable and should be used as a hardware identifier for this specific towable.
    pub vin: String,
}
impl std::fmt::Display for Towable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}