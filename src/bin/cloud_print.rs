use cloud_print::{config::AppConfig, dirs::Dirs, logger, main_executors};
use log::info;
use std::{
    sync::{Arc, Mutex},
    thread,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Logs init
    logger::setup_logger(String::from("cloud_print.log"))?;
    // Load config file
    let app_config = AppConfig::from_config_file();
    let v1 = app_config.clone();
    let v2 = app_config.clone();
    // Generate work directory
    Dirs::generate_working_dir(&app_config);

    let should_close = Arc::new(Mutex::new(false));
    let should_close1 = Arc::clone(&should_close);
    let should_close2 = Arc::clone(&should_close);

    // Spawn threads
    let h1 = thread::Builder::new()
        .name("printer-manager".to_string())
        .spawn(move || {
            info!("init printer manager thread");
            let should_close1 = Arc::clone(&should_close1);
            match main_executors::printer_manager(v2, Some(&should_close1)) {
                Ok(()) => {
                    println!("[printer-manager] finished succesfuly");
                }
                Err(e) => {
                    println!("[printer-manager] error:\n{e:?}");
                    let mut must_close = should_close1.lock().unwrap();
                    *must_close = true;
                }
            }
        });

    let h2 = thread::Builder::new()
        .name("cloud-file-manager".to_string())
        .spawn(move || {
            let should_close2: Arc<Mutex<bool>> = Arc::clone(&should_close2);
            info!("init cloud file manager thread");
            match main_executors::cloud_file_manager(v1, Some(&should_close2)) {
                Ok(()) => {
                    println!("[cloud-file-manager] finished succesfuly");
                }
                Err(e) => {
                    println!("[cloud-file-manager] error:\n{e:?}");
                    let mut must_close = should_close2.lock().unwrap();
                    *must_close = true;
                }
            }
        });

    for handle in [h1, h2] {
        handle.unwrap().join().unwrap()
    }

    Ok(())
}
