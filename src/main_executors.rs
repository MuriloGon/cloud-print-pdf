use crate::{config::AppConfig, message::Message, message_manager::MessageManager};
use crate::{message_manager::FileType, printer_manager::PrinterManager};
use log::{error, info};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::{error::Error, fs, path::Path};
use websocket::ClientBuilder;

pub fn cloud_file_manager(
    app_config: AppConfig,
    should_close_mutex: Option<&Arc<Mutex<bool>>>,
) -> Result<(), Box<dyn Error>> {
    let msg_manager = MessageManager::new(&app_config);
    let ws_url = format!(
        "{}?eventId={}&context={}&pwd={}",
        &app_config.ws_url,
        &app_config.ws_context_id,
        &app_config.ws_context_name,
        &app_config.ws_context_pwd
    );
    info!("Websocket url={}", &ws_url);

    let mut builder = match ClientBuilder::new(&ws_url) {
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

    let handle_error =
        |erro_phrase: String, original_body: String, error: Box<dyn std::error::Error>| {
            error!("{}: {:?}", erro_phrase, error);
            let mut error_msg = Message::default();
            error_msg.set_error(error.to_string(), Some(original_body));
            let file_name = format!("{}.json", &error_msg.id.clone().unwrap());
            msg_manager.save_error_message(&file_name, &error_msg);
        };

    for message_result in client.incoming_messages() {
        if let Some(should_close) = should_close_mutex {
            let should_close = *should_close.lock().unwrap();
            info!("[cloud_file_manager] should_close={should_close})");
            if should_close {
                info!("[cloud_file_manager] should_close={should_close}. Finishing thread");
                break;
            } else {
                info!("[cloud_file_manager] should_close={should_close}. Not finishing yet");
            }
        }

        let message = match message_result {
            Ok(v) => v,
            Err(e) => {
                error!("Websocket error: {:?}", e);
                panic!("Websocket error: {:?}", e);
            }
        };

        let msg_opt: Option<String> = match message {
            websocket::OwnedMessage::Text(t) => Some(t),
            e => {
                error!("Websocket Error \n{:?}", e);
                panic!();
            }
        };

        if let Some(msg_string) = msg_opt {
            let msg_result: Result<Message, serde_json::Error> =
                serde_json::from_str(&msg_string.clone());
            let mut msg = match msg_result {
                Ok(v) => v,
                Err(e) => {
                    handle_error(
                        "Error parsing ws Message".to_string(),
                        msg_string.clone(),
                        Box::new(e),
                    );
                    continue;
                }
            };

            let response = match reqwest::blocking::get(&msg.pdf_url) {
                Ok(v) => v,
                Err(e) => {
                    handle_error(
                        "Error downloading file".to_string(),
                        msg_string.clone(),
                        Box::new(e),
                    );
                    continue;
                }
            };

            let pdf_file_path = Path::new("wdir")
                .join("pdfs")
                .join(format!("{}.pdf", msg.id.clone().unwrap()));

            if let Err(e) = response.error_for_status_ref() {
                handle_error(
                    "Error on request".to_string(),
                    msg_string.clone(),
                    Box::new(e),
                );
                continue;
            };

            let pdf_bytes = match response.bytes() {
                Ok(v) => v,
                Err(e) => {
                    handle_error(
                        "Error parsing bytes".to_string(),
                        msg_string.clone(),
                        Box::new(e),
                    );
                    continue;
                }
            };

            msg.set_downloaded_file_path(pdf_file_path.to_string_lossy().to_string());

            match fs::write(&pdf_file_path, pdf_bytes) {
                Ok(_) => {
                    info!(
                        "Pdf saved successfuly at {}",
                        &pdf_file_path.to_string_lossy().to_string()
                    )
                }
                Err(e) => {
                    handle_error(
                        "Error saving file config locally".to_string(),
                        msg_string.clone(),
                        Box::new(e),
                    );
                    continue;
                }
            }

            let pending_json_path = Path::new("wdir")
                .join("pending")
                .join(format!("{}.json", &msg.id.as_ref().unwrap()));

            match fs::write(
                &pending_json_path,
                serde_json::to_string_pretty(&msg).unwrap(),
            ) {
                Ok(_) => {
                    info!(
                        "Pending file saved successfuly at {}",
                        &pending_json_path.to_string_lossy().to_string()
                    )
                }
                Err(e) => {
                    handle_error(
                        "Error saving message file locally".to_string(),
                        msg_string.clone(),
                        Box::new(e),
                    );
                    continue;
                }
            }
        }
    }

    Ok(())
}

pub fn printer_manager(
    app_config: AppConfig,
    should_close_mutex: Option<&Arc<Mutex<bool>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let printer = PrinterManager::new(&app_config);

    let listed_dir_path = Path::new(&app_config.root_path)
        .join(&app_config.work_dir_name)
        .join("pending");

    // Notification that list all directory change on {workdir}/pending
    if let Err(error) = watch(listed_dir_path, &printer, app_config, should_close_mutex) {
        log::error!("Error: {error:?}");
    }

    Ok(())
}

fn watch<P: AsRef<Path>>(
    path: P,
    printer: &PrinterManager,
    app_config: AppConfig,
    should_close_mutex: Option<&Arc<Mutex<bool>>>,
) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;
    for res in rx {
        if let Some(should_close) = should_close_mutex {
            let should_close = *should_close.lock().unwrap();
            info!("[printer_manager] should_close={should_close})");
            if should_close {
                info!("[printer_manager] should_close={should_close}. Finishing thread");
                break;
            } else {
                info!("[printer_manager] should_close={should_close}. Not finishing yet");
            }
        }

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
