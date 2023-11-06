use std::{
    fs,
    path::{self, PathBuf},
    process::Command,
};

use log::{error, info};

use crate::{config::AppConfig, message::Message};

#[derive(Debug)]
pub struct Ticket {
    pub event_id: String,
    pub ticket_id: String,
}

#[derive(Clone)]
pub struct Printer {
    app_config: AppConfig,
}

#[derive(Debug)]
pub enum PrintStatus {
    Ok,
    Error,
    Pending,
}

impl Printer {
    pub fn new(app_config: &AppConfig) -> Printer {
        Printer {
            app_config: app_config.clone(),
        }
    }

    pub fn import_message(&self, path: &String) -> Result<Message, Message> {
        let path_file: PathBuf = [
            &self.app_config.work_dir_name,
            &"pending".to_string(),
            &path,
        ]
        .iter()
        .collect();

        let json_string_file = match std::fs::read_to_string(&path_file) {
            Ok(x) => {
                info!("message imported for path={}", x);
                info!("\n{:?}", path_file);
                x
            }
            Err(x) => {
                error!("Error parsing pending json: {:?}", x);
                panic!()
            }
        };

        let message_result: Result<Message, serde_json::Error> =
            serde_json::from_str(json_string_file.as_str());

        info!("try parse string");
        info!("{}", json_string_file.as_str());

        let msg = match message_result {
            Ok(msg) => {
                info!("message parsed");
                info!("{:?}", path_file);
                msg
            }
            Err(x) => {
                error!("Error parsing message: {:?}", x.to_string());
                panic!()
            }
        };

        match msg.clone().is_valid() {
            Ok(()) => Ok(msg),
            Err(msg_with_error) => Err(msg_with_error),
        }
    }

    pub fn move_message(
        self,
        status: PrintStatus,
        file_path: &String,
        message: &Message,
    ) -> Result<(), Message> {
        let path_from = path::Path::new(&self.app_config.work_dir_name)
            .join("pending".to_string())
            .join(file_path.to_string());
        let path_to_ok = path::Path::new(&self.app_config.work_dir_name)
            .join("ok".to_string())
            .join(file_path.to_string());
        let path_to_error = path::Path::new(&self.app_config.work_dir_name)
            .join("error".to_string())
            .join(file_path.to_string());

        let log_info = |from: &String, to: &String| {
            info!("Moved from {} to {}", from, to);
        };
        let log_error = |from: &String, to: &String, err: String| {
            error!("Moved from {} to {}", from, to);
        };

        match status {
            PrintStatus::Ok => {
                let rename_result = fs::rename(&path_from, &path_to_ok);
                match rename_result {
                    Ok(_) => {
                        log_info(
                            &path_from.to_string_lossy().to_string(),
                            &path_to_ok.to_string_lossy().to_string(),
                        );
                    }
                    Err(e) => {
                        log_error(
                            &path_from.to_string_lossy().to_string(),
                            &path_to_ok.to_string_lossy().to_string(),
                            e.to_string(),
                        );
                    }
                }
            }
            PrintStatus::Error => {
                let rename_result = fs::rename(&path_from, &path_to_error);
                match rename_result {
                    Ok(_) => {
                        log_info(
                            &path_from.to_string_lossy().to_string(),
                            &path_to_ok.to_string_lossy().to_string(),
                        );
                    }
                    Err(e) => {
                        log_error(
                            &path_from.to_string_lossy().to_string(),
                            &path_to_ok.to_string_lossy().to_string(),
                            e.to_string(),
                        );
                    }
                }
            }
            PrintStatus::Pending => {}
        }

        Ok(())
    }

    pub fn print_ticket(self, message: Message) {
        log::info!("Message printed {:?}", message);
        let executable_path = path::Path::new(&self.app_config.printer_bin);
        let _result = Command::new(executable_path)
            .args(self.app_config.printer_args)
            .spawn()
            .map_err(|err| info!("deu ruim porque {}", err));
    }
}
