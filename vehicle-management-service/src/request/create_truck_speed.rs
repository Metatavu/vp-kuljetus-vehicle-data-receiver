use serde_json::json;
use crate::model::*;
use crate::FluentRequest;
use serde::{Serialize, Deserialize};
use httpclient::InMemoryResponseExt;
use crate::VehicleManagementServiceClient;
/**You should use this struct via [`VehicleManagementServiceClient::create_truck_speed`].

On request success, this will return a [`()`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTruckSpeedRequest {
    pub id: i64,
    pub speed: f64,
    pub truck_id: String,
}
impl CreateTruckSpeedRequest {}
impl FluentRequest<'_, CreateTruckSpeedRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, CreateTruckSpeedRequest> {
    type Output = httpclient::InMemoryResult<()>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = &format!(
                "/vehicle-management/v1/trucks/{truck_id}/speeds", truck_id = self.params
                .truck_id
            );
            let mut r = self.client.client.post(url);
            r = r.json(json!({ "id" : self.params.id }));
            r = r.json(json!({ "speed" : self.params.speed }));
            r = self.client.authenticate(r);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}