use serde_json::json;
use crate::model::*;
use crate::FluentRequest;
use serde::{Serialize, Deserialize};
use httpclient::InMemoryResponseExt;
use crate::VehicleManagementServiceClientClient;
/**You should use this struct via [`VehicleManagementServiceClientClient::create_truck_driver_card`].

On request success, this will return a [`TruckDriverCard`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTruckDriverCardRequest {
    pub id: String,
    pub truck_id: String,
}
impl CreateTruckDriverCardRequest {}
impl FluentRequest<'_, CreateTruckDriverCardRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, CreateTruckDriverCardRequest> {
    type Output = httpclient::InMemoryResult<TruckDriverCard>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = &format!(
                "/vehicle-management/v1/trucks/{truck_id}/driverCards", truck_id = self
                .params.truck_id
            );
            let mut r = self.client.client.post(url);
            r = r.json(json!({ "id" : self.params.id }));
            r = self.client.authenticate(r);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}