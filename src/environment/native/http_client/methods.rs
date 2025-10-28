//! HTTP method implementations

use super::{client::HttpClient, errors::HttpClientError};

impl HttpClient {
    /// Execute HTTP GET request with resilience patterns
    #[inline]
    pub async fn get(&self, url: &str) -> Result<reqwest::Response, HttpClientError> {
        self.execute_with_resilience(url, || self.client.get(url)).await
    }

    /// Execute HTTP POST request with resilience patterns
    #[inline]
    pub async fn post(&self, url: &str, body: reqwest::Body) -> Result<reqwest::Response, HttpClientError> {
        self.execute_with_resilience(url, || self.client.post(url).body(body.clone())).await
    }

    /// Execute HTTP PUT request with resilience patterns
    #[inline]
    pub async fn put(&self, url: &str, body: reqwest::Body) -> Result<reqwest::Response, HttpClientError> {
        self.execute_with_resilience(url, || self.client.put(url).body(body.clone())).await
    }

    /// Execute HTTP DELETE request with resilience patterns
    #[inline]
    pub async fn delete(&self, url: &str) -> Result<reqwest::Response, HttpClientError> {
        self.execute_with_resilience(url, || self.client.delete(url)).await
    }
}