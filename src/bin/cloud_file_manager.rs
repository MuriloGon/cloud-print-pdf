use cloud_print::{config::AppConfig, dirs::Dirs, logger, main_executors};
use log::{info, warn};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    logger::setup_logger(String::from("cloud_file_manager.log"))?;
    let app_config = AppConfig::from_config_file();
    Dirs::generate_working_dir(&app_config);

    // indefinitely
    loop {
        if let Err(e) = main_executors::cloud_file_manager(app_config.clone(), None) {
            warn!("Some error occured on cloud_file_manager\n{}", e);
            info!("Trying restart the application")
        };
    }
}
