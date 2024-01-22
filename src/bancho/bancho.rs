use serde::Deserialize;
use serde_json::json;
use tracing::error;

#[derive(Debug, Deserialize)]
pub struct MessageResponse {
    pub ok: bool,
    pub message: String
}

pub async fn send_message(user_id: i32, message: String, secret: String) -> Option<MessageResponse>{
    let client = reqwest::Client::new();

    
    let response = client
    .post("https://lisek.world/api/v1/server/message")
    .body(json!({ "message": message, "to": user_id }).to_string())
    .header("X-Key", secret)
    .header("Content-Type", "application/json")
    .send()
    .await;

    if let Err(error) = response {
        error!("Request failed: {}", error);
        return None
    }

    let response = response.unwrap();

    let body = response.json::<MessageResponse>().await;

    if let Err(error) = body {
        error!("Response parsing failed: {}", error);
        return None
    }

    let body = body.unwrap();

    Some(body)
}