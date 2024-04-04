use serde_json::json;
use crate::model::*;
use crate::FluentRequest;
use serde::{Serialize, Deserialize};
use httpclient::InMemoryResponseExt;
use crate::VehicleManagementServiceClientClient;
/**You should use this struct via [`VehicleManagementServiceClientClient::create_truck_location`].

On request success, this will return a [`()`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTruckLocationRequest {
    pub heading: f64,
    pub id: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub timestamp: i64,
    pub truck_id: String,
}
impl CreateTruckLocationRequest {}
pub struct CreateTruckLocationRequired<'a> {
    pub heading: f64,
    pub latitude: f64,
    pub longitude: f64,
    pub timestamp: i64,
    pub truck_id: &'a str,
}
impl<'a> CreateTruckLocationRequired<'a> {}
impl FluentRequest<'_, CreateTruckLocationRequest> {
    ///Set the value of the id field.
    pub fn id(mut self, id: &str) -> Self {
        self.params.id = Some(id.to_owned());
        self
    }
}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, CreateTruckLocationRequest> {
    type Output = httpclient::InMemoryResult<()>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = &format!(
                "/vehicle-management/v1/trucks/{truck_id}/locations", truck_id = self
                .params.truck_id
            );
            let mut r = self.client.client.post(url);
            r = r.json(json!({ "heading" : self.params.heading }));
            if let Some(ref unwrapped) = self.params.id {
                r = r.json(json!({ "id" : unwrapped }));
            }
            r = r.json(json!({ "latitude" : self.params.latitude }));
            r = r.json(json!({ "longitude" : self.params.longitude }));
            r = r.json(json!({ "timestamp" : self.params.timestamp }));
            r = self.client.authenticate(r);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}