use serde_json::json;
use crate::model::*;
use crate::FluentRequest;
use serde::{Serialize, Deserialize};
use httpclient::InMemoryResponseExt;
use crate::VehicleManagementServiceClient;
/**You should use this struct via [`VehicleManagementServiceClient::list_public_trucks`].

On request success, this will return a [`Vec<PublicTruck>`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPublicTrucksRequest {
    pub first: Option<i64>,
    pub max: Option<i64>,
}
impl ListPublicTrucksRequest {}
impl FluentRequest<'_, ListPublicTrucksRequest> {
    ///Set the value of the first field.
    pub fn first(mut self, first: i64) -> Self {
        self.params.first = Some(first);
        self
    }
    ///Set the value of the max field.
    pub fn max(mut self, max: i64) -> Self {
        self.params.max = Some(max);
        self
    }
}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, ListPublicTrucksRequest> {
    type Output = httpclient::InMemoryResult<Vec<PublicTruck>>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = "/vehicle-management/v1/publicTrucks";
            let mut r = self.client.client.get(url);
            r = r.set_query(self.params);
            r = self.client.authenticate(r);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}