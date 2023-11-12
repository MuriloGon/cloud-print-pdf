use cloud_print::{main_executors, config::AppConfig, dirs::Dirs};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_config = AppConfig::from_config_file();
    Dirs::generate_working_dir(&app_config);

    main_executors::printer_manager(app_config)?;
    Ok(())
}
