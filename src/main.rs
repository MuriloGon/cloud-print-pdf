mod config;
mod logger;
mod message;
mod printer;

use config::AppConfig;
use log::info;
use path_abs::PathInfo;
use printer::Printer;

use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{self, Path, PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    logger::setup_logger()?;

    let app_config = AppConfig::from_config_file();
    let printer = printer::Printer::new(&app_config);

    let listen_dir_path = path::Path::new(&app_config.root_path)
        .join(&app_config.work_dir_name)
        .join("pending");

    // Init dirs
    init_directories(&app_config);

    // Notification that list all directory change on {workdir}/pending
    if let Err(error) = watch(listen_dir_path, &printer) {
        log::error!("Error: {error:?}");
    }

    Ok(())
}

fn init_directories(config: &AppConfig) {
    AppConfig::generate_working_dir(config)
}

fn watch<P: AsRef<Path>>(path: P, printer: &Printer) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;
    for res in rx {
        match res {
            Ok(event) => {
                if event.kind.is_create() {
                    let path: PathBuf = event.paths.iter().collect();
                    info!(
                        "New file add to pending. eventType={:?}, path={}",
                        event.kind,
                        path.to_string_lossy()
                    );
                    let meupath = path.file_name().unwrap().to_str().unwrap().to_string();
                    handle_print(&printer, &meupath);
                }
            }
            Err(error) => log::error!("Error: {error:?}"),
        }
    }
    Ok(())
}

fn handle_print(printer: &Printer, file_name: &String) {
    let printer = printer.clone();
    let path_string = file_name;
    let msg_result = printer.import_message(&path_string);
    match &msg_result {
        Ok(msg_ok) => {
            let print_result = printer.print_file(&msg_ok);
            if let Err(error_msg) = print_result {
                let mut msg = msg_ok.clone();
                msg.set_error(error_msg);
                printer.update_message(&path_string, &msg);
                printer.move_message(printer::PrintStatus::Error, &path_string);
                return;
            }
            let mut msg = msg_ok.clone();
            msg.set_successful();
            printer.update_message(&path_string, &msg);
            printer.move_message(printer::PrintStatus::Ok, &path_string);
            return;
        }
        Err(msg_error) => {
            let msg = msg_error.clone();
            printer.update_message(&path_string, &msg);
            printer.move_message(printer::PrintStatus::Error, &path_string);
            return;
        }
    }
}
