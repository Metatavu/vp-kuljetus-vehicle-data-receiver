use serde_json::json;
use crate::model::*;
use crate::FluentRequest;
use serde::{Serialize, Deserialize};
use httpclient::InMemoryResponseExt;
use crate::VehicleManagementServiceClientClient;
/**You should use this struct via [`VehicleManagementServiceClientClient::receive_telematic_data`].

On request success, this will return a [`()`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiveTelematicDataRequest {
    pub imei: String,
    pub latitude: f64,
    pub longitude: f64,
    pub speed: f64,
    pub timestamp: i64,
    pub vin: String,
}
impl ReceiveTelematicDataRequest {}
pub struct ReceiveTelematicDataRequired<'a> {
    pub imei: &'a str,
    pub latitude: f64,
    pub longitude: f64,
    pub speed: f64,
    pub timestamp: i64,
    pub vin: &'a str,
}
impl<'a> ReceiveTelematicDataRequired<'a> {}
impl FluentRequest<'_, ReceiveTelematicDataRequest> {}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, ReceiveTelematicDataRequest> {
    type Output = httpclient::InMemoryResult<()>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = &format!(
                "/vehicle-management/v1/telematics/{vin}", vin = self.params.vin
            );
            let mut r = self.client.client.post(url);
            r = r.json(json!({ "imei" : self.params.imei }));
            r = r.json(json!({ "latitude" : self.params.latitude }));
            r = r.json(json!({ "longitude" : self.params.longitude }));
            r = r.json(json!({ "speed" : self.params.speed }));
            r = r.json(json!({ "timestamp" : self.params.timestamp }));
            r = self.client.authenticate(r);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}