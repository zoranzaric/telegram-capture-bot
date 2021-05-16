use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Capture {
    id: Option<u32>,
    content: String,
    created_at: String,
    processed_at: Option<String>,
}
