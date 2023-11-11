use cloud_print::{logger, message::Message};
use log::{error, info};
use std::{error::Error, fs, path};
use websocket::ClientBuilder;

fn main() -> Result<(), Box<dyn Error>> {
    let _ = logger::setup_logger(String::from("cloud_file_man.log"));

    let app_config = cloud_print::config::AppConfig::from_config_file();
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

    for message in client.incoming_messages() {
        let msg_opt: Option<String> = match message.unwrap() {
            websocket::OwnedMessage::Text(t) => Some(t),
            _ => None,
        };

        if let Some(msg_string) = msg_opt {
            let msg_result: Result<Message, serde_json::Error> = serde_json::from_str(&msg_string);
            let msg = match msg_result {
                Ok(v) => v,
                Err(e) => {
                  error!("Error parsing ws Message: {:?}", e);
                  let mut error_msg = Message::default();
                  error_msg.set_error(e.to_string());
                  break;
                },
            };

            let file_name = path::Path::new("wdir")
                .join("ok")
                .join(format!("{}.json", &msg.id.as_ref().unwrap()));

            match fs::write(file_name, serde_json::to_string_pretty(&msg).unwrap()) {
                Ok(_) => {
                    info!("File saved successfuly")
                }
                Err(e) => {
                    error!("Error saving file config locally: {:?}", e)
                }
            }
        }
    }

    Ok(())
}
