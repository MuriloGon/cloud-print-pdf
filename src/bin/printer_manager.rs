use cloud_print::{config::AppConfig, dirs::Dirs, logger, main_executors};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    logger::setup_logger(String::from("printer_manager.log"))?;
    let app_config = AppConfig::from_config_file();
    Dirs::generate_working_dir(&app_config);

    main_executors::printer_manager(app_config, None)?;
    Ok(())
}
