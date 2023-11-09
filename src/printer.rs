use std::{
    fs::{self, File},
    path::{self, PathBuf},
    process::Command,
};

use std::io::Read;

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

        let mut f = loop {
            match File::open(&path_file) {
                Ok(file) => break file,

                // If the file is busy try again until be able to open the file.
                // TODO: Possible infinite loop, maybe make sense se a counter
                Err(e) => match e.raw_os_error() {
                    Some(32) => continue,
                    _ => {
                        panic!("unexpected file open error {:?}", e);
                    }
                },
            };
        };

        let mut data = vec![];
        match f.read_to_end(&mut data) {
            Ok(_v) => {}
            Err(er) => {
                info!("reading to the end: {:?}", er);
                panic!();
            }
        };

        let json_string_file = String::from_utf8(data).unwrap();

        let message_result: Result<Message, serde_json::Error> =
            serde_json::from_str(json_string_file.as_str());

        info!("try parse string");
        info!("{}", json_string_file.as_str());

        match message_result {
            Ok(msg) => {
                info!("message parsed");
                info!("{:?}", path_file);
                Ok(msg)
            }
            Err(x) => {
                error!("Error parsing message: {:?}", x.to_string());
                let msg = Message {
                    error: Some(x.to_string()),
                    is_valid: false,
                    pdf_url: "".to_string(),
                    printed_at: None,
                };
                Err(msg)
            }
        }
    }

    pub fn move_message(&self, status: PrintStatus, file_path: &String) {
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
            error!("Moved from {} to {}. Error={}", from, to, err);
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
                            &path_to_error.to_string_lossy().to_string(),
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
                            &path_to_error.to_string_lossy().to_string(),
                        );
                    }
                    Err(e) => {
                        log_error(
                            &path_from.to_string_lossy().to_string(),
                            &path_to_error.to_string_lossy().to_string(),
                            e.to_string(),
                        );
                    }
                }
            }
        };
    }

    pub fn update_message(&self, file_path: &String, message: &Message) {
        let path_file: PathBuf = [
            &self.app_config.work_dir_name,
            &"pending".to_string(),
            &file_path,
        ]
        .iter()
        .collect();

        let ser_result = serde_json::to_string_pretty(&message);
        let out_string = match ser_result {
            Ok(v) => v,
            Err(e) => {
                error!("Error on serializing msg: {}", e);
                panic!();
            }
        };

        let result_write = fs::write(path_file, out_string);
        match result_write {
            Ok(_) => {
                info!("Message serialized successfuly")
            }
            Err(e) => {
                error!("Error on saving json msg: {}", e);
                panic!();
            }
        }
    }

    pub fn print_file(&self, message: &Message) -> Result<(), String> {
        log::info!("Message printed {:?}", message);
        let executable_path = path::Path::new(&self.app_config.printer_bin);
        let command_result = Command::new(executable_path)
            .args(&self.app_config.printer_args)
            .spawn()
            .map_err(|err| err.to_string());

        match command_result {
            Ok(_child) => {
                info!("Command executed successfuly");
                Ok(())
            }
            Err(e) => {
                info!("Command failed because: {}", e);
                Err(e)
            }
        }
    }
}
