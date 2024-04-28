use serde::Deserialize;
use serde_json::json;
use tracing::{error, info};

#[derive(Debug, Deserialize)]
pub struct MessageResponse {
    pub ok: bool,
    pub message: Option<String>
}

pub async fn send_message(user_id: i32, message: String, secret: String) -> Option<MessageResponse>{
    let client = reqwest::Client::new();

    
    let response = client
    .post("https://c.lisek.world/api/v2/bancho/notification")
    .body(json!({ "message": message, "message_type": "pm", "target": user_id.to_string(), "key": secret }).to_string())
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