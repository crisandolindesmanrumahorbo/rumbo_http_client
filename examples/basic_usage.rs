use rumbo_http_client::{HttpClient, HttpMethod};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // GET request example
    println!("Making GET request...");
    let response = HttpClient::fetch(
        HttpMethod::GET,
        "http://httpbin.org/get".to_string(),
        None::<()>,
    )
    .await?;

    println!("Status: {}", response.status);
    println!("Success: {}", response.is_success());

    if let Some(content_type) = response.header("content-type") {
        println!("Content-Type: {}", content_type);
    }

    if let Some(body) = &response.body {
        println!("Response body: {}", body);
    }

    // POST request example
    println!("\nMaking POST request...");
    let post_data = json!({
        "name": "John Doe",
        "email": "john@example.com"
    });

    let post_response = HttpClient::fetch(
        HttpMethod::POST,
        "http://httpbin.org/post".to_string(),
        Some(post_data),
    )
    .await?;

    println!("POST Status: {}", post_response.status);
    if let Some(body) = &post_response.body {
        println!("POST Response: {}", body);
    }

    Ok(())
}
