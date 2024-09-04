use anyhow::{Context, Result};
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;

pub struct HttpRequest {
  pub url: String,
  pub headers: HashMap<String, String>,
}

pub struct HttpResponse {
  pub status: u16,
  pub headers: HashMap<String, String>,
  pub body: Vec<u8>,
}

#[async_trait]
pub trait HttpClient: Send + Sync + std::fmt::Debug {
  async fn get(&self, url: &str, headers: &HashMap<String, String>) -> Result<HttpResponse>;
}

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

    let response = req.send().await.context("Failed to send request")?;
    let status = response.status().as_u16();
    let headers = response
      .headers()
      .iter()
      .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
      .collect();
    let body = response.bytes().await.context("Failed to read response bytes")?.to_vec();

    Ok(HttpResponse { status, headers, body })
  }
}
