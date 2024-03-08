use serde_json::json;
use crate::model::*;
use crate::FluentRequest;
use serde::{Serialize, Deserialize};
use httpclient::InMemoryResponseExt;
use crate::VehicleManagementServiceClient;
/**You should use this struct via [`VehicleManagementServiceClient::list_vehicles`].

On request success, this will return a [`Vec<Vehicle>`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListVehiclesRequest {
    pub archived: Option<bool>,
    pub first: Option<i64>,
    pub max: Option<i64>,
    pub truck_id: Option<String>,
}
impl ListVehiclesRequest {}
impl FluentRequest<'_, ListVehiclesRequest> {
    ///Set the value of the archived field.
    pub fn archived(mut self, archived: bool) -> Self {
        self.params.archived = Some(archived);
        self
    }
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
    ///Set the value of the truck_id field.
    pub fn truck_id(mut self, truck_id: &str) -> Self {
        self.params.truck_id = Some(truck_id.to_owned());
        self
    }
}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, ListVehiclesRequest> {
    type Output = httpclient::InMemoryResult<Vec<Vehicle>>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = "/v1/vehicles";
            let mut r = self.client.client.get(url);
            r = r.set_query(self.params);
            r = self.client.authenticate(r);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}