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


/// struct for typed errors of method [`list_public_trucks`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ListPublicTrucksError {
    DefaultResponse(models::Error),
    UnknownValue(serde_json::Value),
}


/// Lists public info about each truck.
pub async fn list_public_trucks(configuration: &configuration::Configuration, params: ListPublicTrucksParams) -> Result<Vec<models::PublicTruck>, Error<ListPublicTrucksError>> {
    let local_var_configuration = configuration;

    // unbox the parameters
    let vin = params.vin;
    let first = params.first;
    let max = params.max;


    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/vehicle-management/v1/publicTrucks", local_var_configuration.base_path);
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
