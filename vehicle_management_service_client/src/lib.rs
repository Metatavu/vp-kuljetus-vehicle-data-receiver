//! [`VehicleManagementServiceClientClient`](struct.VehicleManagementServiceClientClient.html) is the main entry point for this library.
//!
//! Library created with [`libninja`](https://www.libninja.com).
#![allow(non_camel_case_types)]
#![allow(unused)]
pub mod model;
pub mod request;
pub use httpclient::{Error, Result, InMemoryResponseExt};
use std::sync::{Arc, OnceLock};
use std::borrow::Cow;
use crate::model::*;
use base64::{Engine, engine::general_purpose::STANDARD_NO_PAD};
static SHARED_HTTPCLIENT: OnceLock<httpclient::Client> = OnceLock::new();
pub fn default_http_client() -> httpclient::Client {
    httpclient::Client::new()
        .base_url(
            std::env::var("VEHICLE_MANAGEMENT_SERVICE_CLIENT_BASE_URL")
                .expect(
                    "Missing environment variable VEHICLE_MANAGEMENT_SERVICE_CLIENT_BASE_URL",
                )
                .as_str(),
        )
}
/// Use this method if you want to add custom middleware to the httpclient.
/// It must be called before any requests are made, otherwise it will have no effect.
/// Example usage:
///
/// ```
/// init_http_client(default_http_client()
///     .with_middleware(..)
/// );
/// ```
pub fn init_http_client(init: httpclient::Client) {
    let _ = SHARED_HTTPCLIENT.set(init);
}
fn shared_http_client() -> Cow<'static, httpclient::Client> {
    Cow::Borrowed(SHARED_HTTPCLIENT.get_or_init(default_http_client))
}
#[derive(Clone)]
pub struct FluentRequest<'a, T> {
    pub(crate) client: &'a VehicleManagementServiceClientClient,
    pub params: T,
}
pub struct VehicleManagementServiceClientClient {
    client: Cow<'static, httpclient::Client>,
    authentication: VehicleManagementServiceClientAuth,
}
impl VehicleManagementServiceClientClient {
    pub fn from_env() -> Self {
        Self {
            client: shared_http_client(),
            authentication: VehicleManagementServiceClientAuth::from_env(),
        }
    }
    pub fn with_auth(authentication: VehicleManagementServiceClientAuth) -> Self {
        Self {
            client: shared_http_client(),
            authentication,
        }
    }
    pub fn new_with(
        client: httpclient::Client,
        authentication: VehicleManagementServiceClientAuth,
    ) -> Self {
        Self {
            client: Cow::Owned(client),
            authentication,
        }
    }
}
impl VehicleManagementServiceClientClient {
    pub(crate) fn authenticate<'a>(
        &self,
        mut r: httpclient::RequestBuilder<'a>,
    ) -> httpclient::RequestBuilder<'a> {
        match &self.authentication {
            VehicleManagementServiceClientAuth::ApiKeyAuth { x_api_key } => {
                r = r.header("X-API-Key", x_api_key);
            }
        }
        r
    }
    /**Updates driver cards

Updates single driver card.*/
    pub fn update_driver_card(
        &self,
        driver_card_id: &str,
    ) -> FluentRequest<'_, request::UpdateDriverCardRequest> {
        FluentRequest {
            client: self,
            params: request::UpdateDriverCardRequest {
                driver_card_id: driver_card_id.to_owned(),
                truck_vin: None,
            },
        }
    }
    /**List PublicTrucks.

Lists public info about each truck.*/
    pub fn list_public_trucks(
        &self,
    ) -> FluentRequest<'_, request::ListPublicTrucksRequest> {
        FluentRequest {
            client: self,
            params: request::ListPublicTrucksRequest {
                first: None,
                max: None,
            },
        }
    }
    ///Receives telematic data entry
    pub fn receive_telematic_data(
        &self,
        args: request::ReceiveTelematicDataRequired,
    ) -> FluentRequest<'_, request::ReceiveTelematicDataRequest> {
        FluentRequest {
            client: self,
            params: request::ReceiveTelematicDataRequest {
                imei: args.imei.to_owned(),
                latitude: args.latitude,
                longitude: args.longitude,
                speed: args.speed,
                timestamp: args.timestamp,
                vin: args.vin.to_owned(),
            },
        }
    }
}
pub enum VehicleManagementServiceClientAuth {
    ApiKeyAuth { x_api_key: String },
}
impl VehicleManagementServiceClientAuth {
    pub fn from_env() -> Self {
        Self::ApiKeyAuth {
            x_api_key: std::env::var("VEHICLE_MANAGEMENT_SERVICE_CLIENT_X_API_KEY")
                .expect(
                    "Environment variable VEHICLE_MANAGEMENT_SERVICE_CLIENT_X_API_KEY is not set.",
                ),
        }
    }
}