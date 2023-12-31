use std::{path::Path, process::Command};

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
        log::info!("Message used on current print: \n{}", &message);

        let pdf_string = message.clone().pdf_local_path.unwrap();

        let pdf_path = Path::new(&pdf_string);

        let executable_path = Path::new(&self.app_config.root_path)
        .join(&self.app_config.printer_bin);

        let mut args: Vec<String> = Vec::new();
        args.push(String::from("-print-to"));
        args.push(format!("{}", &self.app_config.printer_name));
        args.push(String::from("-print-settings"));
        args.push(format!("{}", &self.app_config.printer_settings));
        args.push(format!("{}", &pdf_path.to_string_lossy().to_string()));

        let command_executed = format!("{} {}", &executable_path.to_string_lossy(), args.join(" "));
        info!("Command that will be executed:\n\"{}\"", command_executed);

        let command_result = Command::new(executable_path)
            .args(&args)
            .output()
            .map_err(|err| err.to_string());

        info!("{command_result:?}");

        match command_result {
            Ok(output) => {
                info!("Command executed successfuly\n{}", std::str::from_utf8(&output.stdout).unwrap());
                Ok(())
            }
            Err(e) => {
                info!("Command failed because: {}", e);
                Err(e)
            }
        }
    }
}
