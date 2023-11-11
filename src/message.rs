use std::{fs::File, io::Read, path::PathBuf};

use chrono::{DateTime, Utc};
use log::{error, info};
use serde::{Deserialize, Serialize};

use serde_json::{json, Value};

use crate::config::AppConfig;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Message {
    #[serde(default = "default_id")]
    pub id: Option<String>,
    pub pdf_url: String,
    pub pdf_local_path: Option<String>,
    pub is_valid: Option<bool>,
    pub printed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub context: Option<Value>,
}

impl Default for Message {
    fn default() -> Self {
        Self {
            id: default_id(),
            pdf_url: String::from("invalid-value"),
            pdf_local_path: None,
            is_valid: None,
            printed_at: None,
            error: None,
            context: None,
        }
    }
}

impl Message {
    pub fn set_error(&mut self, error: String) {
        self.is_valid = Some(false);
        self.error = Some(error);
    }

    pub fn set_successful(&mut self) {
        self.is_valid = Some(true);
        self.printed_at = Some(chrono::Utc::now());
    }

    pub fn merge(self, new_msg: Message) -> Message {
        Message {
            id: self.id,
            pdf_url: new_msg.pdf_url,
            pdf_local_path: self.pdf_local_path.or(new_msg.pdf_local_path),
            is_valid: self.is_valid.or(new_msg.is_valid),
            printed_at: self.printed_at.or(new_msg.printed_at),
            error: self.error.or(new_msg.error),
            context: self.context.or(new_msg.context),
        }
    }

    pub fn import_with_path(path: &String, config: AppConfig) -> Result<Message, Message> {
        let path_file: PathBuf = [config.work_dir_name, String::from("pending"), path.clone()]
            .iter()
            .collect();

        let mut f = loop {
            match File::open(&path_file) {
                Ok(file) => break file,

                // If the file is busy try again until be able to open the file.
                // TODO: Possible infinite loop, maybe make sense se a counter
                Err(e) => match e.raw_os_error() {
                    Some(32) => continue,
                    _ => {
                        panic!("Unexpected file open error {:?}", e);
                    }
                },
            };
        };

        let mut data = vec![];
        match f.read_to_end(&mut data) {
            Ok(_v) => {}
            Err(er) => {
                info!("Error reading to the end: {:?}", er);
                panic!();
            }
        };

        let json_string_file = String::from_utf8(data).unwrap();

        let message_result: Result<Message, serde_json::Error> =
            serde_json::from_str(json_string_file.as_str());

        info!("Trying parse string:\n{}", json_string_file.as_str());

        match message_result {
            Ok(msg) => {
                info!("Message parsed");
                info!("{:?}", path_file);
                Ok(msg)
            }
            Err(x) => {
                error!("Error parsing message: {:?}", x.to_string());

                let custom_error_msg = json!({
                  "message": x.to_string(),
                  "payload": json_string_file
                })
                .to_string();

                let msg = Message {
                    error: Some(custom_error_msg),
                    is_valid: None,
                    pdf_url: "Invalid URL".to_string(),
                    printed_at: None,
                    pdf_local_path: None,
                    context: None,
                    ..Message::default()
                };
                Err(msg)
            }
        }
    }
}

fn default_id() -> Option<String> {
    let mut ulid = ulid::Generator::new();
    let id = Some(ulid.generate().unwrap().to_string());
    id
}

#[cfg(test)]
mod message_tests {
    use super::*;

    #[test]
    pub fn is_valid() {
        let new_message = Message {
            pdf_url: "".to_string(),
            pdf_local_path: None,
            error: None,
            printed_at: None,
            is_valid: None,
            context: None,
            ..Default::default()
        };

        assert_eq!(new_message.error, None);
    }
}
