//! [`VehicleManagementServiceClient`](struct.VehicleManagementServiceClient.html) is the main entry point for this library.
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
mod serde;
static SHARED_HTTPCLIENT: OnceLock<httpclient::Client> = OnceLock::new();
pub fn default_http_client() -> httpclient::Client {
    httpclient::Client::new()
        .base_url(
            std::env::var("VEHICLE_MANAGEMENT_SERVICE_BASE_URL")
                .expect(
                    "Missing environment variable VEHICLE_MANAGEMENT_SERVICE_BASE_URL",
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
    pub(crate) client: &'a VehicleManagementServiceClient,
    pub params: T,
}
pub struct VehicleManagementServiceClient {
    client: Cow<'static, httpclient::Client>,
    authentication: VehicleManagementServiceAuth,
}
impl VehicleManagementServiceClient {
    pub fn from_env() -> Self {
        Self {
            client: shared_http_client(),
            authentication: VehicleManagementServiceAuth::from_env(),
        }
    }
    pub fn with_auth(authentication: VehicleManagementServiceAuth) -> Self {
        Self {
            client: shared_http_client(),
            authentication,
        }
    }
    pub fn new_with(
        client: httpclient::Client,
        authentication: VehicleManagementServiceAuth,
    ) -> Self {
        Self {
            client: Cow::Owned(client),
            authentication,
        }
    }
}
impl VehicleManagementServiceClient {
    pub(crate) fn authenticate<'a>(
        &self,
        mut r: httpclient::RequestBuilder<'a>,
    ) -> httpclient::RequestBuilder<'a> {
        match &self.authentication {
            VehicleManagementServiceAuth::BearerAuth { bearer_auth } => {
                r = r.bearer_auth(bearer_auth);
            }
            VehicleManagementServiceAuth::ApiKeyAuth { x_api_key } => {
                r = r.header("X-API-Key", x_api_key);
            }
        }
        r
    }
    /**Replies with pong

Replies ping with pong*/
    pub fn ping(&self) -> FluentRequest<'_, request::PingRequest> {
        FluentRequest {
            client: self,
            params: request::PingRequest {},
        }
    }
    /**List Trucks.

Lists Trucks.*/
    pub fn list_trucks(&self) -> FluentRequest<'_, request::ListTrucksRequest> {
        FluentRequest {
            client: self,
            params: request::ListTrucksRequest {
                archived: None,
                first: None,
                max: None,
                plate_number: None,
            },
        }
    }
    /**Create truck

Create new truck*/
    pub fn create_truck(
        &self,
        plate_number: &str,
        type_: &str,
        vin: &str,
    ) -> FluentRequest<'_, request::CreateTruckRequest> {
        FluentRequest {
            client: self,
            params: request::CreateTruckRequest {
                active_vehicle_id: None,
                archived_at: None,
                created_at: None,
                creator_id: None,
                id: None,
                last_modifier_id: None,
                modified_at: None,
                name: None,
                plate_number: plate_number.to_owned(),
                type_: type_.to_owned(),
                vin: vin.to_owned(),
            },
        }
    }
    /**Find a truck.

Finds a truck by id.*/
    pub fn find_truck(
        &self,
        truck_id: &str,
    ) -> FluentRequest<'_, request::FindTruckRequest> {
        FluentRequest {
            client: self,
            params: request::FindTruckRequest {
                truck_id: truck_id.to_owned(),
            },
        }
    }
    /**Updates trucks

Updates single truck*/
    pub fn update_truck(
        &self,
        args: request::UpdateTruckRequired,
    ) -> FluentRequest<'_, request::UpdateTruckRequest> {
        FluentRequest {
            client: self,
            params: request::UpdateTruckRequest {
                active_vehicle_id: None,
                archived_at: None,
                created_at: None,
                creator_id: None,
                id: None,
                last_modifier_id: None,
                modified_at: None,
                name: None,
                plate_number: args.plate_number.to_owned(),
                truck_id: args.truck_id.to_owned(),
                type_: args.type_.to_owned(),
                vin: args.vin.to_owned(),
            },
        }
    }
    /**Deletes truck

Deletes truck. For non-production use. Returns forbidden in production environment.*/
    pub fn delete_truck(
        &self,
        truck_id: &str,
    ) -> FluentRequest<'_, request::DeleteTruckRequest> {
        FluentRequest {
            client: self,
            params: request::DeleteTruckRequest {
                truck_id: truck_id.to_owned(),
            },
        }
    }
    /**List Towables.

Lists Towables.*/
    pub fn list_towables(&self) -> FluentRequest<'_, request::ListTowablesRequest> {
        FluentRequest {
            client: self,
            params: request::ListTowablesRequest {
                archived: None,
                first: None,
                max: None,
                plate_number: None,
            },
        }
    }
    /**Create towable

Create new towable*/
    pub fn create_towable(
        &self,
        plate_number: &str,
        type_: &str,
        vin: &str,
    ) -> FluentRequest<'_, request::CreateTowableRequest> {
        FluentRequest {
            client: self,
            params: request::CreateTowableRequest {
                archived_at: None,
                created_at: None,
                creator_id: None,
                id: None,
                last_modifier_id: None,
                modified_at: None,
                name: None,
                plate_number: plate_number.to_owned(),
                type_: type_.to_owned(),
                vin: vin.to_owned(),
            },
        }
    }
    /**Find a towable.

Finds a towable by id.*/
    pub fn find_towable(
        &self,
        towable_id: &str,
    ) -> FluentRequest<'_, request::FindTowableRequest> {
        FluentRequest {
            client: self,
            params: request::FindTowableRequest {
                towable_id: towable_id.to_owned(),
            },
        }
    }
    /**Updates towables

Updates single towable*/
    pub fn update_towable(
        &self,
        args: request::UpdateTowableRequired,
    ) -> FluentRequest<'_, request::UpdateTowableRequest> {
        FluentRequest {
            client: self,
            params: request::UpdateTowableRequest {
                archived_at: None,
                created_at: None,
                creator_id: None,
                id: None,
                last_modifier_id: None,
                modified_at: None,
                name: None,
                plate_number: args.plate_number.to_owned(),
                towable_id: args.towable_id.to_owned(),
                type_: args.type_.to_owned(),
                vin: args.vin.to_owned(),
            },
        }
    }
    /**Deletes towable

Deletes towable. For non-production use. Returns forbidden response in production environment.*/
    pub fn delete_towable(
        &self,
        towable_id: &str,
    ) -> FluentRequest<'_, request::DeleteTowableRequest> {
        FluentRequest {
            client: self,
            params: request::DeleteTowableRequest {
                towable_id: towable_id.to_owned(),
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
    /**List Vehicles.

Lists Vehicles.*/
    pub fn list_vehicles(&self) -> FluentRequest<'_, request::ListVehiclesRequest> {
        FluentRequest {
            client: self,
            params: request::ListVehiclesRequest {
                archived: None,
                first: None,
                max: None,
                truck_id: None,
            },
        }
    }
    /**Create vehicle

Create new vehicle. Vehicles are the history of the combinations of towables behind a truck. When a vehicle structure needs to be updated, a new vehicle with updated structure should be created. This updates the active vehicle for the truck and archives the previous one.*/
    pub fn create_vehicle(
        &self,
        towable_ids: &[&str],
        truck_id: &str,
    ) -> FluentRequest<'_, request::CreateVehicleRequest> {
        FluentRequest {
            client: self,
            params: request::CreateVehicleRequest {
                archived_at: None,
                created_at: None,
                creator_id: None,
                id: None,
                last_modifier_id: None,
                modified_at: None,
                towable_ids: towable_ids.iter().map(|&x| x.to_owned()).collect(),
                truck_id: truck_id.to_owned(),
            },
        }
    }
    /**Find a vehicle.

Finds a vehicle by id.*/
    pub fn find_vehicle(
        &self,
        vehicle_id: &str,
    ) -> FluentRequest<'_, request::FindVehicleRequest> {
        FluentRequest {
            client: self,
            params: request::FindVehicleRequest {
                vehicle_id: vehicle_id.to_owned(),
            },
        }
    }
    /**Deletes vehicle

Deletes vehicle. For non-production use. Returns forbidden response in production environment.*/
    pub fn delete_vehicle(
        &self,
        vehicle_id: &str,
    ) -> FluentRequest<'_, request::DeleteVehicleRequest> {
        FluentRequest {
            client: self,
            params: request::DeleteVehicleRequest {
                vehicle_id: vehicle_id.to_owned(),
            },
        }
    }
}
pub enum VehicleManagementServiceAuth {
    BearerAuth { bearer_auth: String },
    ApiKeyAuth { x_api_key: String },
}
impl VehicleManagementServiceAuth {
    pub fn from_env() -> Self {
        Self::BearerAuth {
            bearer_auth: std::env::var("VEHICLE_MANAGEMENT_SERVICE_BEARER_AUTH")
                .expect(
                    "Environment variable VEHICLE_MANAGEMENT_SERVICE_BEARER_AUTH is not set.",
                ),
        }
    }
}