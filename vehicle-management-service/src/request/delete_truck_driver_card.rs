use serde_json::json;
use crate::model::*;
use crate::FluentRequest;
use serde::{Serialize, Deserialize};
use httpclient::InMemoryResponseExt;
use crate::VehicleManagementServiceClient;
/**You should use this struct via [`VehicleManagementServiceClient::delete_truck_driver_card`].

On request success, this will return a [`()`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteTruckDriverCardRequest {
    pub driver_card_id: String,
    pub truck_id: String,
}
impl DeleteTruckDriverCardRequest {}
impl FluentRequest<'_, DeleteTruckDriverCardRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, DeleteTruckDriverCardRequest> {
    type Output = httpclient::InMemoryResult<()>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = &format!(
                "/vehicle-management/v1/trucks/{truck_id}/driverCards/{driver_card_id}",
                driver_card_id = self.params.driver_card_id, truck_id = self.params
                .truck_id
            );
            let mut r = self.client.client.delete(url);
            r = r.set_query(self.params);
            r = self.client.authenticate(r);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}