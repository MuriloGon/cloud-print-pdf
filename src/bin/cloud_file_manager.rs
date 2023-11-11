use cloud_print::{config::AppConfig, logger, message::Message, message_manager::MessageManager};
use log::{error, info};
use std::{error::Error, fs, path::Path};
use websocket::ClientBuilder;

fn main() -> Result<(), Box<dyn Error>> {
    let _ = logger::setup_logger(String::from("cloud_file_man.log"));

    let app_config = AppConfig::from_config_file();
    let msg_manager = MessageManager::new(&app_config);
    let ws_url = format!(
        "{}?eventId={}&context={}&pwd={}",
        &app_config.ws_url,
        &app_config.ws_context_id,
        &app_config.ws_context_name,
        &app_config.ws_context_pwd
    );
    info!("Websocket url={}", &ws_url);

    let mut builder = match ClientBuilder::new(&ws_url) {
        Ok(x) => {
            info!("Websocket URL is OK");
            x
        }
        Err(e) => {
            error!("Error parsing ws url: {:?}", e);
            panic!();
        }
    };

    let mut client = match builder.connect(None) {
        Ok(c) => {
            info!("Client successfuly connected");
            c
        }
        Err(e) => {
            error!("Error connecting ws client {:?}", e.to_string());
            panic!();
        }
    };

    let handle_error =
        |erro_phrase: String, original_body: String, error: Box<dyn std::error::Error>| {
            error!("{}: {:?}", erro_phrase, error);
            let mut error_msg = Message::default();
            error_msg.set_error(error.to_string(), Some(original_body));
            let file_name = format!("{}.json", &error_msg.id.clone().unwrap());
            msg_manager.save_error_message(&file_name, &error_msg);
        };

    for message in client.incoming_messages() {
        let msg_opt: Option<String> = match message.unwrap() {
            websocket::OwnedMessage::Text(t) => Some(t),
            _ => None,
        };

        if let Some(msg_string) = msg_opt {
            let msg_result: Result<Message, serde_json::Error> =
                serde_json::from_str(&msg_string.clone());
            let mut msg = match msg_result {
                Ok(v) => v,
                Err(e) => {
                    handle_error(
                        "Error parsing ws Message".to_string(),
                        msg_string.clone(),
                        Box::new(e),
                    );
                    continue;
                }
            };

            let response = match reqwest::blocking::get(&msg.pdf_url) {
                Ok(v) => v,
                Err(e) => {
                    handle_error(
                        "Error downloading file".to_string(),
                        msg_string.clone(),
                        Box::new(e),
                    );
                    continue;
                }
            };

            let pdf_file_path = Path::new("wdir")
                .join("pdfs")
                .join(format!("{}.pdf", msg.id.clone().unwrap()));

            if let Err(e) = response.error_for_status_ref() {
                handle_error(
                    "Error on request".to_string(),
                    msg_string.clone(),
                    Box::new(e),
                );
                continue;
            };

            let pdf_bytes = match response.bytes() {
                Ok(v) => v,
                Err(e) => {
                    handle_error(
                        "Error parsing bytes".to_string(),
                        msg_string.clone(),
                        Box::new(e),
                    );
                    continue;
                }
            };

            msg.set_downloaded_file_path(pdf_file_path.to_string_lossy().to_string());

            match fs::write(&pdf_file_path, pdf_bytes) {
                Ok(_) => {
                    info!(
                        "Pdf saved successfuly at {}",
                        &pdf_file_path.to_string_lossy().to_string()
                    )
                }
                Err(e) => {
                    handle_error(
                        "Error saving file config locally".to_string(),
                        msg_string.clone(),
                        Box::new(e),
                    );
                    continue;
                }
            }

            let pending_json_path = Path::new("wdir")
                .join("pending")
                .join(format!("{}.json", &msg.id.as_ref().unwrap()));

            match fs::write(
                &pending_json_path,
                serde_json::to_string_pretty(&msg).unwrap(),
            ) {
                Ok(_) => {
                    info!(
                        "Pending file saved successfuly at {}",
                        &pending_json_path.to_string_lossy().to_string()
                    )
                }
                Err(e) => {
                    handle_error(
                        "Error saving message file locally".to_string(),
                        msg_string.clone(),
                        Box::new(e),
                    );
                    continue;
                }
            }
        }
    }

    Ok(())
}
