/*
 * VP-Kuljetus Vehicle Management Services
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 1.0.0
 * 
 * Generated by: https://openapi-generator.tech
 */


use reqwest;

use crate::{apis::ResponseContent, models};
use super::{Error, configuration};

/// struct for passing parameters to the method [`create_vehicle`]
#[derive(Clone, Debug)]
pub struct CreateVehicleParams {
    /// Payload
    pub vehicle: models::Vehicle
}

/// struct for passing parameters to the method [`find_towable`]
#[derive(Clone, Debug)]
pub struct FindTowableParams {
    /// towables id
    pub towable_id: String
}

/// struct for passing parameters to the method [`find_truck`]
#[derive(Clone, Debug)]
pub struct FindTruckParams {
    /// trucks id
    pub truck_id: String
}

/// struct for passing parameters to the method [`find_vehicle`]
#[derive(Clone, Debug)]
pub struct FindVehicleParams {
    /// vehicles id
    pub vehicle_id: String
}

/// struct for passing parameters to the method [`list_drive_states`]
#[derive(Clone, Debug)]
pub struct ListDriveStatesParams {
    /// truck id
    pub truck_id: String,
    /// Filter results by driver ID
    pub driver_id: Option<String>,
    /// Filter results by driver state
    pub state: Option<Vec<models::TruckDriveStateEnum>>,
    /// Filter results after given date-time
    pub after: Option<String>,
    /// Filter results before given date-time
    pub before: Option<String>,
    /// First result.
    pub first: Option<i32>,
    /// Max results.
    pub max: Option<i32>
}

/// struct for passing parameters to the method [`list_public_trucks`]
#[derive(Clone, Debug)]
pub struct ListPublicTrucksParams {
    /// Filter results by vin
    pub vin: Option<String>,
    /// First result.
    pub first: Option<i32>,
    /// Max results.
    pub max: Option<i32>
}

/// struct for passing parameters to the method [`list_towables`]
#[derive(Clone, Debug)]
pub struct ListTowablesParams {
    /// Filter results by plate number
    pub plate_number: Option<String>,
    /// Filter results by archived status
    pub archived: Option<bool>,
    /// First result.
    pub first: Option<i32>,
    /// Max results.
    pub max: Option<i32>
}

/// struct for passing parameters to the method [`list_trucks`]
#[derive(Clone, Debug)]
pub struct ListTrucksParams {
    /// Filter results by plate number
    pub plate_number: Option<String>,
    /// Filter results by archived status
    pub archived: Option<bool>,
    /// First result.
    pub first: Option<i32>,
    /// Max results.
    pub max: Option<i32>
}

/// struct for passing parameters to the method [`list_vehicles`]
#[derive(Clone, Debug)]
pub struct ListVehiclesParams {
    /// Filter results by truck id
    pub truck_id: Option<String>,
    /// Filter results by archived status
    pub archived: Option<bool>,
    /// First result.
    pub first: Option<i32>,
    /// Max results.
    pub max: Option<i32>
}


/// struct for typed errors of method [`create_vehicle`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateVehicleError {
    DefaultResponse(models::Error),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`find_towable`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FindTowableError {
    DefaultResponse(models::Error),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`find_truck`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FindTruckError {
    DefaultResponse(models::Error),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`find_vehicle`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FindVehicleError {
    DefaultResponse(models::Error),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`list_drive_states`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ListDriveStatesError {
    DefaultResponse(models::Error),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`list_public_trucks`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ListPublicTrucksError {
    DefaultResponse(models::Error),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`list_towables`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ListTowablesError {
    DefaultResponse(models::Error),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`list_trucks`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ListTrucksError {
    DefaultResponse(models::Error),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`list_vehicles`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ListVehiclesError {
    DefaultResponse(models::Error),
    UnknownValue(serde_json::Value),
}


/// Create new vehicle. Vehicles are the history of the combinations of towables behind a truck. When a vehicle structure needs to be updated, a new vehicle with updated structure should be created. This updates the active vehicle for the truck and archives the previous one. 
pub async fn create_vehicle(configuration: &configuration::Configuration, params: CreateVehicleParams) -> Result<models::Vehicle, Error<CreateVehicleError>> {
    let local_var_configuration = configuration;

    // unbox the parameters
    let vehicle = params.vehicle;


    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/v1/vehicles", local_var_configuration.base_path);
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::POST, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };
    local_var_req_builder = local_var_req_builder.json(&vehicle);

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<CreateVehicleError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Finds a towable by id.
pub async fn find_towable(configuration: &configuration::Configuration, params: FindTowableParams) -> Result<models::Towable, Error<FindTowableError>> {
    let local_var_configuration = configuration;

    // unbox the parameters
    let towable_id = params.towable_id;


    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/v1/towables/{towableId}", local_var_configuration.base_path, towableId=crate::apis::urlencode(towable_id));
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<FindTowableError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Finds a truck by id.
pub async fn find_truck(configuration: &configuration::Configuration, params: FindTruckParams) -> Result<models::Truck, Error<FindTruckError>> {
    let local_var_configuration = configuration;

    // unbox the parameters
    let truck_id = params.truck_id;


    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/v1/trucks/{truckId}", local_var_configuration.base_path, truckId=crate::apis::urlencode(truck_id));
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<FindTruckError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Finds a vehicle by id.
pub async fn find_vehicle(configuration: &configuration::Configuration, params: FindVehicleParams) -> Result<models::Vehicle, Error<FindVehicleError>> {
    let local_var_configuration = configuration;

    // unbox the parameters
    let vehicle_id = params.vehicle_id;


    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/v1/vehicles/{vehicleId}", local_var_configuration.base_path, vehicleId=crate::apis::urlencode(vehicle_id));
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<FindVehicleError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Lists drive states for truck.
pub async fn list_drive_states(configuration: &configuration::Configuration, params: ListDriveStatesParams) -> Result<Vec<models::TruckDriveState>, Error<ListDriveStatesError>> {
    let local_var_configuration = configuration;

    // unbox the parameters
    let truck_id = params.truck_id;
    let driver_id = params.driver_id;
    let state = params.state;
    let after = params.after;
    let before = params.before;
    let first = params.first;
    let max = params.max;


    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/v1/trucks/{truckId}/driveStates", local_var_configuration.base_path, truckId=crate::apis::urlencode(truck_id));
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_str) = driver_id {
        local_var_req_builder = local_var_req_builder.query(&[("driverId", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = state {
        local_var_req_builder = match "multi" {
            "multi" => local_var_req_builder.query(&local_var_str.into_iter().map(|p| ("state".to_owned(), p.to_string())).collect::<Vec<(std::string::String, std::string::String)>>()),
            _ => local_var_req_builder.query(&[("state", &local_var_str.into_iter().map(|p| p.to_string()).collect::<Vec<String>>().join(",").to_string())]),
        };
    }
    if let Some(ref local_var_str) = after {
        local_var_req_builder = local_var_req_builder.query(&[("after", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = before {
        local_var_req_builder = local_var_req_builder.query(&[("before", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = first {
        local_var_req_builder = local_var_req_builder.query(&[("first", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = max {
        local_var_req_builder = local_var_req_builder.query(&[("max", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<ListDriveStatesError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Lists public info about each truck.
pub async fn list_public_trucks(configuration: &configuration::Configuration, params: ListPublicTrucksParams) -> Result<Vec<models::PublicTruck>, Error<ListPublicTrucksError>> {
    let local_var_configuration = configuration;

    // unbox the parameters
    let vin = params.vin;
    let first = params.first;
    let max = params.max;


    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/v1/publicTrucks", local_var_configuration.base_path);
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_str) = vin {
        local_var_req_builder = local_var_req_builder.query(&[("vin", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = first {
        local_var_req_builder = local_var_req_builder.query(&[("first", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = max {
        local_var_req_builder = local_var_req_builder.query(&[("max", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(ref local_var_apikey) = local_var_configuration.api_key {
        let local_var_key = local_var_apikey.key.clone();
        let local_var_value = match local_var_apikey.prefix {
            Some(ref local_var_prefix) => format!("{} {}", local_var_prefix, local_var_key),
            None => local_var_key,
        };
        local_var_req_builder = local_var_req_builder.header("X-API-Key", local_var_value);
    };
    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<ListPublicTrucksError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Lists Towables.
pub async fn list_towables(configuration: &configuration::Configuration, params: ListTowablesParams) -> Result<Vec<models::Towable>, Error<ListTowablesError>> {
    let local_var_configuration = configuration;

    // unbox the parameters
    let plate_number = params.plate_number;
    let archived = params.archived;
    let first = params.first;
    let max = params.max;


    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/v1/towables", local_var_configuration.base_path);
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_str) = plate_number {
        local_var_req_builder = local_var_req_builder.query(&[("plateNumber", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = archived {
        local_var_req_builder = local_var_req_builder.query(&[("archived", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = first {
        local_var_req_builder = local_var_req_builder.query(&[("first", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = max {
        local_var_req_builder = local_var_req_builder.query(&[("max", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<ListTowablesError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Lists Trucks.
pub async fn list_trucks(configuration: &configuration::Configuration, params: ListTrucksParams) -> Result<Vec<models::Truck>, Error<ListTrucksError>> {
    let local_var_configuration = configuration;

    // unbox the parameters
    let plate_number = params.plate_number;
    let archived = params.archived;
    let first = params.first;
    let max = params.max;


    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/v1/trucks", local_var_configuration.base_path);
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_str) = plate_number {
        local_var_req_builder = local_var_req_builder.query(&[("plateNumber", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = archived {
        local_var_req_builder = local_var_req_builder.query(&[("archived", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = first {
        local_var_req_builder = local_var_req_builder.query(&[("first", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = max {
        local_var_req_builder = local_var_req_builder.query(&[("max", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<ListTrucksError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Lists Vehicles.
pub async fn list_vehicles(configuration: &configuration::Configuration, params: ListVehiclesParams) -> Result<Vec<models::Vehicle>, Error<ListVehiclesError>> {
    let local_var_configuration = configuration;

    // unbox the parameters
    let truck_id = params.truck_id;
    let archived = params.archived;
    let first = params.first;
    let max = params.max;


    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/v1/vehicles", local_var_configuration.base_path);
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_str) = truck_id {
        local_var_req_builder = local_var_req_builder.query(&[("truckId", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = archived {
        local_var_req_builder = local_var_req_builder.query(&[("archived", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = first {
        local_var_req_builder = local_var_req_builder.query(&[("first", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = max {
        local_var_req_builder = local_var_req_builder.query(&[("max", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<ListVehiclesError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

