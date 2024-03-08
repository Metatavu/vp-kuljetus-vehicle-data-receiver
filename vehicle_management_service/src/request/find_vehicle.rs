use serde_json::json;
use crate::model::*;
use crate::FluentRequest;
use serde::{Serialize, Deserialize};
use httpclient::InMemoryResponseExt;
use crate::VehicleManagementServiceClient;
/**You should use this struct via [`VehicleManagementServiceClient::find_vehicle`].

On request success, this will return a [`Vehicle`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindVehicleRequest {
    pub vehicle_id: String,
}
impl FindVehicleRequest {}
impl FluentRequest<'_, FindVehicleRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, FindVehicleRequest> {
    type Output = httpclient::InMemoryResult<Vehicle>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = &format!(
                "/v1/vehicles/{vehicle_id}", vehicle_id = self.params.vehicle_id
            );
            let mut r = self.client.client.get(url);
            r = r.set_query(self.params);
            r = self.client.authenticate(r);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}