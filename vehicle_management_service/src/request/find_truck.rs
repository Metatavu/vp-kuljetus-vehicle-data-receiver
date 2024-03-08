use serde_json::json;
use crate::model::*;
use crate::FluentRequest;
use serde::{Serialize, Deserialize};
use httpclient::InMemoryResponseExt;
use crate::VehicleManagementServiceClient;
/**You should use this struct via [`VehicleManagementServiceClient::find_truck`].

On request success, this will return a [`Truck`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindTruckRequest {
    pub truck_id: String,
}
impl FindTruckRequest {}
impl FluentRequest<'_, FindTruckRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, FindTruckRequest> {
    type Output = httpclient::InMemoryResult<Truck>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = &format!("/v1/trucks/{truck_id}", truck_id = self.params.truck_id);
            let mut r = self.client.client.get(url);
            r = r.set_query(self.params);
            r = self.client.authenticate(r);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}