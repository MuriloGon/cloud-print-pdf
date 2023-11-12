use cloud_print::{config::AppConfig, main_executors, dirs::Dirs};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_config = AppConfig::from_config_file();
    Dirs::generate_working_dir(&app_config);

    main_executors::cloud_file_manager(app_config)?;
    Ok(())
}
