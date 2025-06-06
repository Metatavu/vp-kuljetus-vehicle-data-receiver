/*
 * VP-Kuljetus Vehicle Management Services
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 1.0.0
 * 
 * Generated by: https://openapi-generator.tech
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// Error : Error object
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Error {
    #[serde(rename = "status")]
    pub status: i32,
    #[serde(rename = "message")]
    pub message: String,
}

impl Error {
    /// Error object
    pub fn new(status: i32, message: String) -> Error {
        Error {
            status,
            message,
        }
    }
}

