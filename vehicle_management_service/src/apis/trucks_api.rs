/*
 * Vehicle Management Services (vehicle-data-receiver)
 *
 * Vehicle Management Services (vehicle-data-receiver)
 *
 * The version of the OpenAPI document: 1.0.0
 * 
 * Generated by: https://openapi-generator.tech
 */


use reqwest;

use crate::{apis::ResponseContent, models};
use super::{Error, configuration};

/// struct for passing parameters to the method [`create_drive_state`]
#[derive(Clone, Debug)]
pub struct CreateDriveStateParams {
    /// truck id
    pub truck_id: String,
    /// Payload
    pub truck_drive_state: models::TruckDriveState
}

/// struct for passing parameters to the method [`create_truck_driver_card`]
#[derive(Clone, Debug)]
pub struct CreateTruckDriverCardParams {
    /// truck ID
    pub truck_id: String,
    /// Payload
    pub truck_driver_card: models::TruckDriverCard
}

/// struct for passing parameters to the method [`create_truck_location`]
#[derive(Clone, Debug)]
pub struct CreateTruckLocationParams {
    /// truck id
    pub truck_id: String,
    /// Payload
    pub truck_location: models::TruckLocation
}

/// struct for passing parameters to the method [`create_truck_speed`]
#[derive(Clone, Debug)]
pub struct CreateTruckSpeedParams {
    /// truck id
    pub truck_id: String,
    /// Payload
    pub truck_speed: models::TruckSpeed
}

/// struct for passing parameters to the method [`delete_truck_driver_card`]
#[derive(Clone, Debug)]
pub struct DeleteTruckDriverCardParams {
    /// truck ID
    pub truck_id: String,
    /// driver card ID
    pub driver_card_id: String
}


/// struct for typed errors of method [`create_drive_state`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateDriveStateError {
    DefaultResponse(models::Error),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`create_truck_driver_card`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateTruckDriverCardError {
    DefaultResponse(models::Error),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`create_truck_location`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateTruckLocationError {
    DefaultResponse(models::Error),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`create_truck_speed`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateTruckSpeedError {
    DefaultResponse(models::Error),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`delete_truck_driver_card`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DeleteTruckDriverCardError {
    DefaultResponse(models::Error),
    UnknownValue(serde_json::Value),
}


/// Create new drive state for truck
pub async fn create_drive_state(configuration: &configuration::Configuration, params: CreateDriveStateParams) -> Result<(), Error<CreateDriveStateError>> {
    let local_var_configuration = configuration;

    // unbox the parameters
    let truck_id = params.truck_id;
    let truck_drive_state = params.truck_drive_state;


    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/vehicle-management/v1/trucks/{truckId}/driveStates", local_var_configuration.base_path, truckId=crate::apis::urlencode(truck_id));
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::POST, local_var_uri_str.as_str());

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
    local_var_req_builder = local_var_req_builder.json(&truck_drive_state);

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        Ok(())
    } else {
        let local_var_entity: Option<CreateDriveStateError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Create new truck driver card
pub async fn create_truck_driver_card(configuration: &configuration::Configuration, params: CreateTruckDriverCardParams) -> Result<models::TruckDriverCard, Error<CreateTruckDriverCardError>> {
    let local_var_configuration = configuration;

    // unbox the parameters
    let truck_id = params.truck_id;
    let truck_driver_card = params.truck_driver_card;


    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/vehicle-management/v1/trucks/{truckId}/driverCards", local_var_configuration.base_path, truckId=crate::apis::urlencode(truck_id));
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::POST, local_var_uri_str.as_str());

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
    local_var_req_builder = local_var_req_builder.json(&truck_driver_card);

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<CreateTruckDriverCardError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Create new truck location. Used by vehicle data receiver to send truck location data.
pub async fn create_truck_location(configuration: &configuration::Configuration, params: CreateTruckLocationParams) -> Result<(), Error<CreateTruckLocationError>> {
    let local_var_configuration = configuration;

    // unbox the parameters
    let truck_id = params.truck_id;
    let truck_location = params.truck_location;


    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/vehicle-management/v1/trucks/{truckId}/locations", local_var_configuration.base_path, truckId=crate::apis::urlencode(truck_id));
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::POST, local_var_uri_str.as_str());

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
    local_var_req_builder = local_var_req_builder.json(&truck_location);

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        Ok(())
    } else {
        let local_var_entity: Option<CreateTruckLocationError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Create new truck speed. Used by vehicle data receiver to send truck speed data.
pub async fn create_truck_speed(configuration: &configuration::Configuration, params: CreateTruckSpeedParams) -> Result<(), Error<CreateTruckSpeedError>> {
    let local_var_configuration = configuration;

    // unbox the parameters
    let truck_id = params.truck_id;
    let truck_speed = params.truck_speed;


    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/vehicle-management/v1/trucks/{truckId}/speeds", local_var_configuration.base_path, truckId=crate::apis::urlencode(truck_id));
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::POST, local_var_uri_str.as_str());

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
    local_var_req_builder = local_var_req_builder.json(&truck_speed);

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        Ok(())
    } else {
        let local_var_entity: Option<CreateTruckSpeedError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

/// Deletes single truck driver card. Cards are deleted when they are removed from the truck.
pub async fn delete_truck_driver_card(configuration: &configuration::Configuration, params: DeleteTruckDriverCardParams) -> Result<(), Error<DeleteTruckDriverCardError>> {
    let local_var_configuration = configuration;

    // unbox the parameters
    let truck_id = params.truck_id;
    let driver_card_id = params.driver_card_id;


    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/vehicle-management/v1/trucks/{truckId}/driverCards/{driverCardId}", local_var_configuration.base_path, truckId=crate::apis::urlencode(truck_id), driverCardId=crate::apis::urlencode(driver_card_id));
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::DELETE, local_var_uri_str.as_str());

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

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        Ok(())
    } else {
        let local_var_entity: Option<DeleteTruckDriverCardError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}
