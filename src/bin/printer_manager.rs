use std::path::{Path, PathBuf};

use cloud_print::{
    config::AppConfig,
    dirs::Dirs,
    logger,
    message::Message,
    message_manager::{FileType, MessageManager},
    printer_manager::PrinterManager,
};
use log::info;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    logger::setup_logger(String::from("printer_man.log"))?;

    let app_config = AppConfig::from_config_file();
    let printer = PrinterManager::new(&app_config);

    // Init dirs
    Dirs::generate_working_dir(&app_config);

    let listed_dir_path = Path::new(&app_config.root_path)
        .join(&app_config.work_dir_name)
        .join("pending");

    // Notification that list all directory change on {workdir}/pending
    if let Err(error) = watch(listed_dir_path, &printer, app_config) {
        log::error!("Error: {error:?}");
    }

    Ok(())
}

fn watch<P: AsRef<Path>>(
    path: P,
    printer: &PrinterManager,
    app_config: AppConfig,
) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;
    for res in rx {
        match res {
            Ok(event) => {
                if event.kind.is_create() {
                    let path: PathBuf = event.paths.iter().collect();
                    info!(
                        "New file added to pending.\n-eventType={:?}\n-path={}",
                        event.kind,
                        path.to_string_lossy()
                    );
                    let meupath = path.file_name().unwrap().to_str().unwrap().to_string();
                    handle_print(printer, meupath, app_config.clone());
                }
            }
            Err(error) => log::error!("Error: {error:?}"),
        }
    }
    Ok(())
}

fn handle_print(printer: &PrinterManager, file_name: String, app_config: AppConfig) {
    let printer_manager = printer.clone();
    let msg_manager = MessageManager::new(&app_config);
    let msg_result = Message::import_with_path(&file_name, app_config);
    match &msg_result {
        Ok(msg_ok) => {
            let print_result = printer_manager.print_file(&msg_ok);
            if let Err(error_msg) = print_result {
                let mut msg = msg_ok.clone();
                msg.set_error(error_msg, None);
                msg_manager.update_message(&file_name, &msg);
                msg_manager.move_message(FileType::Error, &file_name);
                return;
            }
            let mut msg = msg_ok.clone();
            msg.set_successful();
            msg_manager.update_message(&file_name, &msg);
            msg_manager.move_message(FileType::Ok, &file_name);
            return;
        }
        Err(msg_error) => {
            let msg = msg_error.clone();
            msg_manager.update_message(&file_name, &msg);
            msg_manager.move_message(FileType::Error, &file_name);
            return;
        }
    }
}
