use serde_json::json;
use crate::model::*;
use crate::FluentRequest;
use serde::{Serialize, Deserialize};
use httpclient::InMemoryResponseExt;
use crate::VehicleManagementServiceClient;
/**You should use this struct via [`VehicleManagementServiceClient::create_towable`].

On request success, this will return a [`Towable`].*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTowableRequest {
    pub archived_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub creator_id: Option<String>,
    pub id: Option<String>,
    pub last_modifier_id: Option<String>,
    pub modified_at: Option<chrono::DateTime<chrono::Utc>>,
    pub name: Option<String>,
    pub plate_number: String,
    pub type_: String,
    pub vin: String,
}
impl CreateTowableRequest {}
impl FluentRequest<'_, CreateTowableRequest> {
    ///Set the value of the archived_at field.
    pub fn archived_at(mut self, archived_at: chrono::DateTime<chrono::Utc>) -> Self {
        self.params.archived_at = Some(archived_at);
        self
    }
    ///Set the value of the created_at field.
    pub fn created_at(mut self, created_at: chrono::DateTime<chrono::Utc>) -> Self {
        self.params.created_at = Some(created_at);
        self
    }
    ///Set the value of the creator_id field.
    pub fn creator_id(mut self, creator_id: &str) -> Self {
        self.params.creator_id = Some(creator_id.to_owned());
        self
    }
    ///Set the value of the id field.
    pub fn id(mut self, id: &str) -> Self {
        self.params.id = Some(id.to_owned());
        self
    }
    ///Set the value of the last_modifier_id field.
    pub fn last_modifier_id(mut self, last_modifier_id: &str) -> Self {
        self.params.last_modifier_id = Some(last_modifier_id.to_owned());
        self
    }
    ///Set the value of the modified_at field.
    pub fn modified_at(mut self, modified_at: chrono::DateTime<chrono::Utc>) -> Self {
        self.params.modified_at = Some(modified_at);
        self
    }
    ///Set the value of the name field.
    pub fn name(mut self, name: &str) -> Self {
        self.params.name = Some(name.to_owned());
        self
    }
}
impl<'a> ::std::future::IntoFuture for FluentRequest<'a, CreateTowableRequest> {
    type Output = httpclient::InMemoryResult<Towable>;
    type IntoFuture = ::futures::future::BoxFuture<'a, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let url = "/v1/towables";
            let mut r = self.client.client.post(url);
            if let Some(ref unwrapped) = self.params.archived_at {
                r = r.json(json!({ "archivedAt" : unwrapped }));
            }
            if let Some(ref unwrapped) = self.params.created_at {
                r = r.json(json!({ "createdAt" : unwrapped }));
            }
            if let Some(ref unwrapped) = self.params.creator_id {
                r = r.json(json!({ "creatorId" : unwrapped }));
            }
            if let Some(ref unwrapped) = self.params.id {
                r = r.json(json!({ "id" : unwrapped }));
            }
            if let Some(ref unwrapped) = self.params.last_modifier_id {
                r = r.json(json!({ "lastModifierId" : unwrapped }));
            }
            if let Some(ref unwrapped) = self.params.modified_at {
                r = r.json(json!({ "modifiedAt" : unwrapped }));
            }
            if let Some(ref unwrapped) = self.params.name {
                r = r.json(json!({ "name" : unwrapped }));
            }
            r = r.json(json!({ "plateNumber" : self.params.plate_number }));
            r = r.json(json!({ "type" : self.params.type_ }));
            r = r.json(json!({ "vin" : self.params.vin }));
            r = self.client.authenticate(r);
            let res = r.await?;
            res.json().map_err(Into::into)
        })
    }
}