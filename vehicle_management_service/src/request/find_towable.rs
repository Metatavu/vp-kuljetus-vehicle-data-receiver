use serde_json::json;
use crate::model::*;
use crate::FluentRequest;
use serde::{Serialize, Deserialize};
use httpclient::InMemoryResponseExt;
use crate::VehicleManagementServiceClient;
/**You should use this struct via [`VehicleManagementServiceClient::find_towable`].

On request success, this will return a [`Towable`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindTowableRequest {
    pub towable_id: String,
}
impl FindTowableRequest {}
impl FluentRequest<'_, FindTowableRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, FindTowableRequest> {
    type Output = httpclient::InMemoryResult<Towable>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = &format!(
                "/v1/towables/{towable_id}", towable_id = self.params.towable_id
            );
            let mut r = self.client.client.get(url);
            r = r.set_query(self.params);
            r = self.client.authenticate(r);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}