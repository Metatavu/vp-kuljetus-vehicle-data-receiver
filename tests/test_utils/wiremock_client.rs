// wiremock_helper.rs
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use reqwest::Url;
use serde_json::{json, Value};

/// A client for interacting with Wiremock, allowing for stubbing and verifying HTTP requests.
/// This client provides methods to create stubs for various HTTP methods and paths,
/// reset mappings, and verify request counts.
#[derive(Clone, Debug)]
pub struct WiremockClient {
    base: Url,
    http: reqwest::Client,
}

/// Implementation of WiremockClient.
/// This struct provides methods to create stubs for HTTP requests, reset mappings,
/// and verify the number of requests made to specific endpoints.
impl WiremockClient {
    /// Creates a new instance of WiremockClient.
    /// # Arguments
    /// * `base` - The base URL for the Wiremock server, e.g., "http://127.0.0.1:8080"
    /// # Returns
    /// A Result containing the WiremockClient instance or an error if the URL parsing fails.
    /// # Errors
    /// Returns an error if the base URL cannot be parsed or if the HTTP client cannot be built.
    /// # Panics
    /// Panics if the URL parsing fails.
    pub fn new(base: impl AsRef<str>) -> Result<Self> {
        let base = Url::parse(base.as_ref()).context("parse wiremock base url")?;
        let http = reqwest::Client::builder().timeout(Duration::from_secs(10)).build()?;
        Ok(Self { base, http })
    }

    /// Adds a stub for a specific HTTP request.
    /// # Arguments
    /// * `method` - The HTTP method (e.g., "GET", "POST").
    /// * `url_path` - The path of the URL to match (e.g., "/hello").
    /// * `status` - The HTTP status code to return (e.g., 200).
    /// * `body_json` - Optional JSON body to return in the response.
    /// * `body_text` - Optional text body to return in the response.
    /// # Returns
    /// A Result indicating success or failure of the stub creation.
    /// # Errors
    /// Returns an error if the stub creation fails, such as if the request to Wiremock fails or if the response status is not successful.
    /// # Panics
    /// Panics if the URL cannot be constructed or if the HTTP request fails.
    pub async fn stub(
        &self,
        method: &str,
        url_path: &str,
        status: u16,
        body_json: Option<Value>,
        body_text: Option<&str>,
    ) -> Result<()> {
        let mut response = json!({ "status": status });

        if let Some(v) = body_json {
            response["jsonBody"] = v;
            response["headers"] = json!({ "Content-Type": "application/json" });
        } else if let Some(t) = body_text {
            response["body"] = Value::String(t.to_string());
        }

        let mapping = json!({
            "request": {
                "method": method.to_uppercase(),
                "urlPath": url_path
            },
            "response": response
        });

        let url = self.base.join("/__admin/mappings")?;
        let res = self.http.post(url).json(&mapping).send().await?;
        if !res.status().is_success() {
            return Err(anyhow!("wiremock stub failed: {} {}", res.status(), res.text().await?));
        }
        Ok(())
    }

    /// Removes all mappings from Wiremock.
    /// This method clears all stubs and mappings created in Wiremock.
    /// # Returns
    /// A Result indicating success or failure of the reset operation.
    /// # Errors
    /// Returns an error if the reset operation fails, such as if the request to Wiremock fails or if the response status is not successful.
    /// # Panics
    /// Panics if the request to reset mappings fails.
    pub async fn reset_mappings(&self) -> Result<()> {
        let url = self.base.join("/__admin/mappings")?;
        let res = self.http.delete(url).send().await?;
        if !res.status().is_success() {
            return Err(anyhow!("reset mappings failed: {} {}", res.status(), res.text().await?));
        }
        Ok(())
    }

    /// Removes all stubs and logged requests from Wiremock.
    /// This method clears all stubs and logged requests, effectively resetting Wiremock to a clean state.
    /// # Returns
    /// A Result indicating success or failure of the reset operation.
    /// # Errors
    /// Returns an error if the reset operation fails, such as if the request to Wiremock fails or if the response status is not successful.
    /// # Panics
    /// Panics if the request to reset all mappings fails.
    pub async fn reset_all(&self) -> Result<()> {
        let url = self.base.join("/__admin/reset")?;
        let res = self.http.post(url).send().await?;
        if !res.status().is_success() {
            return Err(anyhow!("reset all failed: {} {}", res.status(), res.text().await?));
        }
        Ok(())
    }

    /// Verify how many times a request happened (exact method + path).
    /// # Arguments
    /// * `method` - The HTTP method to match (e.g., "GET", "POST").
    /// * `url_path` - The path of the URL to match (e.g., "/hello").
    /// # Returns
    /// A Result containing the count of requests made to the specified method and path.
    /// # Errors
    /// Returns an error if the request to count requests fails or if the response status is not successful.
    /// # Panics
    /// Panics if the request to count requests fails or if the response does not contain a valid count.
    pub async fn count_requests(&self, method: &str, url_path: &str) -> Result<u64> {
        // WireMock supports POST /__admin/requests/count with the same matcher structure
        let body = json!({
            "method": method.to_uppercase(),
            "urlPath": url_path
        });
        let url = self.base.join("/__admin/requests/count")?;
        let res = self.http.post(url).json(&body).send().await?;
        if !res.status().is_success() {
            return Err(anyhow!("count requests failed: {} {}", res.status(), res.text().await?));
        }
        let v: Value = res.json().await?;
        let count = v
            .get("count")
            .and_then(Value::as_u64)
            .ok_or_else(|| anyhow!("missing count"))?;
        Ok(count)
    }

    /// Waits for a specified number of requests to be made to a specific method and path.
    /// This method checks the Wiremock server for the number of requests made to the specified endpoint
    /// and waits until the specified count is reached or the timeout is reached.
    /// # Arguments
    /// * `method` - The HTTP method to match (e.g., "GET", "POST").
    /// * `url_path` - The path of the URL to match (e.g., "/hello").
    /// * `expected` - The expected number of requests to wait for.
    /// * `timeout` - The maximum duration to wait for the expected number of requests.
    /// # Returns
    /// A Result containing the count of requests made to the specified method and path.
    /// # Errors
    /// Returns an error if the request to wait for requests fails or if the timeout is reached.
    /// # Panics
    /// Panics if the Wiremock client fails to wait for the specified number of requests.
    pub async fn wait_requests(&self, method: &str, url_path: &str, expected: u64, timeout: Duration) -> Result<u64> {
        let start = std::time::Instant::now();
        loop {
            let count = self.count_requests(method, url_path).await?;
            if count == expected {
                return Ok((count));
            }
            if start.elapsed() > timeout {
                return Err(anyhow!(
                    "timeout waiting for {} {} requests to '{}', got {}",
                    method,
                    expected,
                    url_path,
                    count
                ));
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}
