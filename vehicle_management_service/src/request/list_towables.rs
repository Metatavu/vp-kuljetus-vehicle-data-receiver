use serde_json::json;
use crate::model::*;
use crate::FluentRequest;
use serde::{Serialize, Deserialize};
use httpclient::InMemoryResponseExt;
use crate::VehicleManagementServiceClient;
/**You should use this struct via [`VehicleManagementServiceClient::list_towables`].

On request success, this will return a [`Vec<Towable>`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTowablesRequest {
    pub archived: Option<bool>,
    pub first: Option<i64>,
    pub max: Option<i64>,
    pub plate_number: Option<String>,
}
impl ListTowablesRequest {}
impl FluentRequest<'_, ListTowablesRequest> {
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
    ///Set the value of the plate_number field.
    pub fn plate_number(mut self, plate_number: &str) -> Self {
        self.params.plate_number = Some(plate_number.to_owned());
        self
    }
}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, ListTowablesRequest> {
    type Output = httpclient::InMemoryResult<Vec<Towable>>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = "/v1/towables";
            let mut r = self.client.client.get(url);
            r = r.set_query(self.params);
            r = self.client.authenticate(r);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}