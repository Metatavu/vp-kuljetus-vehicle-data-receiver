use serde::{Serialize, Deserialize};
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DriverCard {
    ///Driver card ID
    #[serde(rename = "driverCardId")]
    pub driver_card_id: String,
    ///Truck's vehicle identification number
    #[serde(rename = "truckVin")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truck_vin: Option<String>,
}
impl std::fmt::Display for DriverCard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}