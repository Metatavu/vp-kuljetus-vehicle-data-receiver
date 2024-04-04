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
                vin: None,
            },
        }
    }
    /**Create truck driver card

Create new truck driver card*/
    pub fn create_truck_driver_card(
        &self,
        id: &str,
        truck_id: &str,
    ) -> FluentRequest<'_, request::CreateTruckDriverCardRequest> {
        FluentRequest {
            client: self,
            params: request::CreateTruckDriverCardRequest {
                id: id.to_owned(),
                truck_id: truck_id.to_owned(),
            },
        }
    }
    /**Deletes truck driver card

Deletes single truck driver card. Cards are deleted when they are removed from the truck.*/
    pub fn delete_truck_driver_card(
        &self,
        driver_card_id: &str,
        truck_id: &str,
    ) -> FluentRequest<'_, request::DeleteTruckDriverCardRequest> {
        FluentRequest {
            client: self,
            params: request::DeleteTruckDriverCardRequest {
                driver_card_id: driver_card_id.to_owned(),
                truck_id: truck_id.to_owned(),
            },
        }
    }
    /**Create truck location

Create new truck location. Used by vehicle data receiver to send truck location data.*/
    pub fn create_truck_location(
        &self,
        args: request::CreateTruckLocationRequired,
    ) -> FluentRequest<'_, request::CreateTruckLocationRequest> {
        FluentRequest {
            client: self,
            params: request::CreateTruckLocationRequest {
                heading: args.heading,
                id: None,
                latitude: args.latitude,
                longitude: args.longitude,
                timestamp: args.timestamp,
                truck_id: args.truck_id.to_owned(),
            },
        }
    }
    /**Create truck speed

Create new truck speed. Used by vehicle data receiver to send truck speed data.*/
    pub fn create_truck_speed(
        &self,
        speed: f64,
        timestamp: i64,
        truck_id: &str,
    ) -> FluentRequest<'_, request::CreateTruckSpeedRequest> {
        FluentRequest {
            client: self,
            params: request::CreateTruckSpeedRequest {
                id: None,
                speed,
                timestamp,
                truck_id: truck_id.to_owned(),
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