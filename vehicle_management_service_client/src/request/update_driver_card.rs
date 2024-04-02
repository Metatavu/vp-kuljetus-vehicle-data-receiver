use serde_json::json;
use crate::model::*;
use crate::FluentRequest;
use serde::{Serialize, Deserialize};
use httpclient::InMemoryResponseExt;
use crate::VehicleManagementServiceClientClient;
/**You should use this struct via [`VehicleManagementServiceClientClient::update_driver_card`].

On request success, this will return a [`DriverCard`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDriverCardRequest {
    pub driver_card_id: String,
    pub truck_vin: Option<String>,
}
impl UpdateDriverCardRequest {}
impl FluentRequest<'_, UpdateDriverCardRequest> {
    ///Set the value of the truck_vin field.
    pub fn truck_vin(mut self, truck_vin: &str) -> Self {
        self.params.truck_vin = Some(truck_vin.to_owned());
        self
    }
}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, UpdateDriverCardRequest> {
    type Output = httpclient::InMemoryResult<DriverCard>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = &format!(
                "/vehicle-management/v1/driverCards/{driver_card_id}", driver_card_id =
                self.params.driver_card_id
            );
            let mut r = self.client.client.put(url);
            if let Some(ref unwrapped) = self.params.truck_vin {
                r = r.json(json!({ "truckVin" : unwrapped }));
            }
            r = self.client.authenticate(r);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}