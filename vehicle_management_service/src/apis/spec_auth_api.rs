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

/// struct for passing parameters to the method [`list_truck_driver_cards`]
#[derive(Clone, Debug)]
pub struct ListTruckDriverCardsParams {
    /// truck ID
    pub truck_id: String
}


/// struct for typed errors of method [`list_truck_driver_cards`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ListTruckDriverCardsError {
    DefaultResponse(models::Error),
    UnknownValue(serde_json::Value),
}


/// Lists truck driver cards. Used to check if a truck has a driver card inserted.
pub async fn list_truck_driver_cards(configuration: &configuration::Configuration, params: ListTruckDriverCardsParams) -> Result<Vec<models::TruckDriverCard>, Error<ListTruckDriverCardsError>> {
    let local_var_configuration = configuration;

    // unbox the parameters
    let truck_id = params.truck_id;


    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/v1/trucks/{truckId}/driverCards", local_var_configuration.base_path, truckId=crate::apis::urlencode(truck_id));
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

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
        let local_var_entity: Option<ListTruckDriverCardsError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

