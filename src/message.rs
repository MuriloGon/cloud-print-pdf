use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use validator::Validate;

#[derive(Debug, Validate, Deserialize, Serialize, Clone)]
pub struct Message {
    #[validate(url)]
    pub pdf_url: String,
    pub is_valid: bool,
    pub printed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

impl Message {
    pub fn new(pdf_url: String) -> Message {
        Message {
            error: None,
            is_valid: true,
            printed_at: None,
            pdf_url,
        }
    }

    pub fn is_valid(self) -> Result<(), Message> {
        match self.validate() {
            Ok(()) => Ok(()),
            Err(e) => {
                let output = e.to_string();
                return Err(Message {
                    error: Some(String::from(output)),
                    is_valid: false,
                    ..self
                });
            }
        }
    }
}

#[cfg(test)]
mod message_tests {
    use super::*;

    #[test]
    pub fn is_valid() {
        let new_message = Message {
            error: None,
            pdf_url: "".to_string(),
            printed_at: None,
            is_valid: true,
        };

        assert_eq!(new_message.error, None);
    }
}
