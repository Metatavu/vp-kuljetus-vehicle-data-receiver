use serde::{Serialize, Deserialize};
///Represent public info of single truck
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PublicTruck {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "plateNumber")]
    pub plate_number: String,
    ///Truck identification number. This is unique for each truck and should be used as a hardware identifier for this specific truck.
    pub vin: String,
}
impl std::fmt::Display for PublicTruck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}