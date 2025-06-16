//! # Mini HTTP Client
//!
//! A minimal HTTP client library for basic GET and POST requests.
//!
//! ## Features
//! - Lightweight and minimal dependencies
//! - Async/await support with tokio
//! - Optional TLS support
//! - JSON serialization support
//!
//! ## Example
//! ```rust
//! use rumbo_http_client::{HttpClient, HttpMethod};
//!
//! #[tokio::main]
//! async fn main() {
//!     let response = HttpClient::fetch(
//!         HttpMethod::GET,
//!         "http://httpbin.org/get".to_string(),
//!         None::<()>
//!     ).await;
//!     
//!     println!("Status: {}", response.status);
//!     if let Some(body) = response.body {
//!         println!("Body: {}", body);
//!     }
//! }
//! ```

use anyhow::{Context, Result};
use serde::Serialize;
use std::collections::HashMap;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
};
use url::Url;

/// HTTP methods supported by the client
#[derive(Debug, Clone)]
pub enum HttpMethod {
    GET,
    POST,
}

/// Errors that can occur during HTTP requests
#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("TLS error: {0}")]
    TlsError(String),
    #[error("Request failed: {0}")]
    RequestFailed(String),
    #[error("Response parsing error: {0}")]
    ResponseParseError(String),
}

/// A minimal HTTP client
pub struct HttpClient;

impl HttpClient {
    /// Perform an HTTP request
    ///
    /// # Arguments
    /// * `method` - HTTP method (GET or POST)
    /// * `url` - Target URL as string
    /// * `body` - Optional request body (for POST requests)
    ///
    /// # Returns
    /// Returns a `Response` containing status, headers, and body
    pub async fn fetch<T: Serialize>(
        method: HttpMethod,
        url: String,
        body: Option<T>,
    ) -> Result<Response, HttpError> {
        let parsed = Url::parse(&url).map_err(|e| HttpError::InvalidUrl(e.to_string()))?;

        let scheme = parsed.scheme();
        let host = parsed
            .host_str()
            .ok_or_else(|| HttpError::InvalidUrl("Missing host".to_string()))?;
        let port = parsed
            .port_or_known_default()
            .ok_or_else(|| HttpError::InvalidUrl("Missing port".to_string()))?;

        let path = parsed.path();
        let full_path = match parsed.query() {
            Some(query) => format!("{}?{}", path, query),
            None => path.to_string(),
        };

        match scheme {
            #[cfg(feature = "tls")]
            "https" => {
                let conn = TcpStream::connect((host, port)).await.map_err(|e| {
                    HttpError::ConnectionFailed(format!("Failed to connect to {}: {}", host, e))
                })?;

                let tls_connector = native_tls::TlsConnector::new()
                    .map_err(|e| HttpError::TlsError(format!("TLS init failed: {}", e)))?;
                let connector = tokio_native_tls::TlsConnector::from(tls_connector);
                let stream = connector
                    .connect(host, conn)
                    .await
                    .map_err(|e| HttpError::TlsError(format!("TLS handshake failed: {}", e)))?;

                Self::make_request(stream, method, host, &full_path, body).await
            }
            #[cfg(not(feature = "tls"))]
            "https" => Err(HttpError::TlsError(
                "TLS support not enabled. Enable 'tls' feature".to_string(),
            )),
            "http" => {
                let stream = TcpStream::connect((host, port)).await.map_err(|e| {
                    HttpError::ConnectionFailed(format!("Failed to connect to {}: {}", host, e))
                })?;

                Self::make_request(stream, method, host, &full_path, body).await
            }
            _ => Err(HttpError::InvalidUrl(format!(
                "Unsupported scheme: {}",
                scheme
            ))),
        }
    }

    async fn make_request<T, S>(
        mut stream: T,
        method: HttpMethod,
        host: &str,
        full_path: &str,
        body: Option<S>,
    ) -> Result<Response, HttpError>
    where
        T: AsyncRead + AsyncWrite + Unpin,
        S: Serialize,
    {
        let request = Self::build_request(method, host, full_path, body)?;

        stream
            .write_all(request.as_bytes())
            .await
            .map_err(|e| HttpError::RequestFailed(format!("Failed to write request: {}", e)))?;

        let response_data = Self::read_response(&mut stream).await?;
        let response_str = String::from_utf8_lossy(&response_data);

        Response::parse(&response_str).map_err(|e| HttpError::ResponseParseError(e.to_string()))
    }

    fn build_request<S: Serialize>(
        method: HttpMethod,
        host: &str,
        full_path: &str,
        body: Option<S>,
    ) -> Result<String, HttpError> {
        match method {
            HttpMethod::GET => Ok(format!(
                "GET {} HTTP/1.1\r\n\
                Host: {}\r\n\
                User-Agent: mini-http-client/0.1.0\r\n\
                Connection: close\r\n\
                \r\n",
                full_path, host
            )),
            HttpMethod::POST => {
                let json_body = if let Some(b) = body {
                    serde_json::to_string(&b).map_err(|e| {
                        HttpError::RequestFailed(format!("JSON serialization failed: {}", e))
                    })?
                } else {
                    String::new()
                };

                Ok(format!(
                    "POST {} HTTP/1.1\r\n\
                    Host: {}\r\n\
                    User-Agent: mini-http-client/0.1.0\r\n\
                    Content-Type: application/json\r\n\
                    Content-Length: {}\r\n\
                    Connection: close\r\n\
                    \r\n\
                    {}",
                    full_path,
                    host,
                    json_body.len(),
                    json_body
                ))
            }
        }
    }

    async fn read_response<T: AsyncRead + Unpin>(stream: &mut T) -> Result<Vec<u8>, HttpError> {
        let mut response = Vec::new();
        let mut buf = [0u8; 4096]; // Increased buffer size for better performance

        loop {
            let n = stream
                .read(&mut buf)
                .await
                .map_err(|e| HttpError::RequestFailed(format!("Failed to read response: {}", e)))?;

            if n == 0 {
                break;
            }
            response.extend_from_slice(&buf[..n]);
        }

        Ok(response)
    }
}

/// HTTP response structure
#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl Response {
    /// Parse HTTP response from string
    pub fn parse(response: &str) -> Result<Self> {
        let mut parts = response.split("\r\n\r\n");
        let headers_section = parts.next().context("Missing headers section")?;

        // Get body (everything after first \r\n\r\n)
        let body = parts
            .next()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        // Parse status line
        let mut lines = headers_section.lines();
        let status_line = lines.next().context("Missing status line")?;
        let status = Self::parse_status_line(status_line)?;

        // Parse headers
        let mut headers = HashMap::new();
        for line in lines {
            if let Some((key, value)) = line.split_once(':') {
                headers.insert(key.trim().to_lowercase(), value.trim().to_string());
            }
        }

        Ok(Response {
            status,
            headers,
            body,
        })
    }

    fn parse_status_line(status_line: &str) -> Result<u16> {
        let parts: Vec<&str> = status_line.split_whitespace().collect();
        if parts.len() < 2 {
            return Err(anyhow::anyhow!("Invalid status line format"));
        }

        parts[1]
            .parse::<u16>()
            .context("Failed to parse status code")
    }

    /// Check if the response status indicates success (2xx)
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// Get a header value by name (case-insensitive)
    pub fn header(&self, name: &str) -> Option<&String> {
        self.headers.get(&name.to_lowercase())
    }
}

// Re-export commonly used types
pub use HttpMethod::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_parsing() {
        let response_str = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 13\r\n\r\n{\"hello\":\"world\"}";

        let response = Response::parse(response_str).unwrap();
        assert_eq!(response.status, 200);
        assert_eq!(
            response.header("content-type"),
            Some(&"application/json".to_string())
        );
        assert_eq!(response.body, Some("{\"hello\":\"world\"}".to_string()));
        assert!(response.is_success());
    }
}
