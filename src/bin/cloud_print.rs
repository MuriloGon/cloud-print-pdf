use cloud_print::{config::AppConfig, dirs::Dirs, logger, main_executors};
use std::{process::exit, thread};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Logs init
    logger::setup_logger(String::from("cloud_print.log"))?;
    // Load config file
    let app_config = AppConfig::from_config_file();
    let v1 = app_config.clone();
    let v2 = app_config.clone();
    // Generate work directory
    Dirs::generate_working_dir(&app_config);
    
    // Spawn threads
    thread::scope(|v| {
        v.spawn(|| match main_executors::printer_manager(v2) {
            Ok(()) => {
                println!("[printer-manager] finished succesfuly");
            }
            Err(e) => {
                println!("[printer-manager] error:\n{e:?}");
                exit(-1);
            }
        });

        v.spawn(|| match main_executors::cloud_file_manager(v1) {
            Ok(()) => {
                println!("[cloud-file-manager] finished succesfuly");
            }
            Err(e) => {
                println!("[cloud-file-manager] error:\n{e:?}");
                exit(-1);
            }
        });
    });

    Ok(())
}
