// Utility classes
// (c) 2024 Ross Younger

use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct GenericError {
    pub error: String,
}
