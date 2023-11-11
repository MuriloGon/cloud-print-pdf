use std::{process::Command, path::Path};

use log::info;

use crate::{config::AppConfig, message::Message};

#[derive(Debug)]
pub struct Ticket {
    pub event_id: String,
    pub ticket_id: String,
}

#[derive(Clone)]
pub struct PrinterManager {
    app_config: AppConfig,
}

impl PrinterManager {
    pub fn new(app_config: &AppConfig) -> PrinterManager {
        PrinterManager {
            app_config: app_config.clone(),
        }
    }

    pub fn print_file(&self, message: &Message) -> Result<(), String> {
        log::info!("Message printed {:?}", message);
        let executable_path = Path::new(&self.app_config.printer_bin);
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
