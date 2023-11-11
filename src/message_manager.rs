use std::{
    fs,
    path::{Path, PathBuf},
};

use log::{error, info};

use crate::{config::AppConfig, message::Message};

pub struct MessageManager {
    app_config: AppConfig,
}

#[derive(Debug)]
pub enum FileType {
    Ok,
    Error,
}

impl MessageManager {
    pub fn new(app_config: &AppConfig) -> MessageManager {
        MessageManager {
            app_config: app_config.clone(),
        }
    }

    pub fn move_message(&self, status: FileType, file_path: &String) {
        let path_from = Path::new(&self.app_config.work_dir_name)
            .join("pending".to_string())
            .join(file_path.to_string());
        let path_to_ok = Path::new(&self.app_config.work_dir_name)
            .join("ok".to_string())
            .join(file_path.to_string());
        let path_to_error = Path::new(&self.app_config.work_dir_name)
            .join("error".to_string())
            .join(file_path.to_string());

        let log_info = |from: &String, to: &String| {
            info!("Moved from {} to {}", from, to);
        };
        let log_error = |from: &String, to: &String, err: String| {
            error!("Moved from {} to {}. Error={}", from, to, err);
        };

        match status {
            FileType::Ok => {
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
            FileType::Error => {
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

    pub fn update_message(&self, file_name: &String, message: &Message) {
        let path_file: PathBuf = [
            &self.app_config.work_dir_name,
            &"pending".to_string(),
            &file_name,
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

    pub fn save_error_message(&self, file_name: &String, message: &Message) {
        let path_file: PathBuf = [
            &self.app_config.work_dir_name,
            &"error".to_string(),
            &file_name,
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
}
