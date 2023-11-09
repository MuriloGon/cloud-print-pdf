use cloud_print::logger;
use log::{error, info};
use std::error::Error;
use websocket::ClientBuilder;

fn main() -> Result<(), Box<dyn Error>> {
    let _ = logger::setup_logger(String::from("cloud_file_man.log"));

    let app_config = cloud_print::config::AppConfig::from_config_file();
    let url = format!(
        "{}?eventId={}&context={}&pwd={}",
        &app_config.ws_url,
        &app_config.ws_context_id,
        &app_config.ws_context_name,
        &app_config.ws_context_pwd
    );
    info!("Websocket url={}", &url);

    let mut builder = match ClientBuilder::new(&url) {
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
        println!("{:?}", &message.unwrap());
    }

    Ok(())
}
