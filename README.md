# Rumbo HTTP Client

A lightweight, minimal HTTP client library for Rust with async support.

## Features

- ✅ Minimal dependencies and small binary size
- ✅ Async/await support with tokio
- ✅ GET and POST requests
- ✅ JSON serialization/deserialization support
- ✅ Optional TLS/HTTPS support
- ✅ Simple, intuitive API
- ✅ Proper error handling

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rumbo_http_client = "0.1.0"

# Enable TLS support for HTTPS requests
rumbo_http_client = { version = "0.1.0", features = ["tls"] }
```

## Quick Start

```rust
use rumbo_http_client::{HttpClient, HttpMethod};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // GET request
    let response = HttpClient::fetch(
        HttpMethod::GET,
        "http://httpbin.org/get".to_string(),
        None::<()>,
    ).await?;

    println!("Status: {}", response.status);
    if let Some(body) = response.body {
        println!("Body: {}", body);
    }

    // POST request with JSON
    let data = json!({"key": "value"});
    let response = HttpClient::fetch(
        HttpMethod::POST,
        "http://httpbin.org/post".to_string(),
        Some(data),
    ).await?;

    Ok(())
}
```

## API Reference

### HttpClient::fetch()

```rust
pub async fn fetch<T: Serialize>(
    method: HttpMethod,
    url: String,
    body: Option<T>
) -> Result<Response, HttpError>
```

### Response

```rust
pub struct Response {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl Response {
    pub fn is_success(&self) -> bool
    pub fn header(&self, name: &str) -> Option<&String>
}
```

## Features

### Default Features

- Basic HTTP support (no TLS)

### Optional Features

- `tls`: Enable HTTPS support using native-tls

## Size Optimization

This library is optimized for minimal size:

- Minimal dependencies
- Optional TLS support
- Compile-time optimizations in `Cargo.toml`
- No unnecessary features

## Examples

Run the example:

```bash
cargo run --example basic_usage --features tls
```

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
