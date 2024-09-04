use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use rspack_plugin_schemes::{HttpClient, HttpRequest, HttpResponse};


#[derive(Debug)]
pub struct ReqwestHttpClient {
  client: Client,
}

impl ReqwestHttpClient {
  pub fn new() -> Self {
    Self {
      client: Client::new(),
    }
  }
}

#[async_trait]
impl HttpClient for ReqwestHttpClient {
  async fn get(&self, url: &str, headers: &HashMap<String, String>) -> Result<HttpResponse> {
    let mut req = self.client.get(url);

    for (key, value) in headers.iter() {
      req = req.header(key, value);
    }

    let response = req.send().await?;
    let status = response.status().as_u16();
    let headers = response
      .headers()
      .iter()
      .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
      .collect();
    let body = response.bytes().await?.to_vec();

    Ok(HttpResponse { status, headers, body })
  }
}
