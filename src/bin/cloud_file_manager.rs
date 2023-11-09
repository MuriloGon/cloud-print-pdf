use cloud_print::logger;
use log::{error, info};
use std::error::Error;
use websocket::{header::Headers, ClientBuilder};

fn main() -> Result<(), Box<dyn Error>> {
    let _ = logger::setup_logger(String::from("cloud_file_man.log"));

    let app_config = cloud_print::config::AppConfig::from_config_file();

    let mut headers = Headers::new();
    headers.append_raw("x-context-id", Vec::from(app_config.context_id.clone()));
    headers.append_raw("x-context-name", Vec::from(app_config.context_name.clone()));

    let mut builder = match ClientBuilder::new(&app_config.websocket_url) {
        Ok(x) => {
            info!("Websock Url is ok");
            x.custom_headers(&headers)
        }
        Err(e) => {
            error!("Error parsing ws url: {:?}", e);
            panic!();
        }
    };

    let client = match builder.connect_insecure() {
        Ok(c) => {
            info!("Clicent successfuly connected");
            c
        }
        Err(e) => {
            error!("Error connecting ws client {:?}", e.to_string());
            panic!();
        }
    };

    let (mut receiver, mut sender) = client.split().unwrap();

    for message in receiver.incoming_messages() {
        println!("{:?}", &message.unwrap());
        // Echo the message back
        // sender.send_message(&message.unwrap()).unwrap();
    }

    Ok(())
}
