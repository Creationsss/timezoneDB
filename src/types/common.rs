use serde::Serialize;

#[derive(Serialize)]
pub struct JsonMessage {
    pub message: String,
}
