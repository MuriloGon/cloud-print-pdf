use std::{fmt::Display, path};

#[derive(PartialEq, Eq, Hash)]
pub enum FileSupported {
    JSON,
    YML,
    YAML,
}

use log::error;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    #[serde(skip)]
    config_file_name: String,
    #[serde(skip)]
    pub root_path: String,
    #[serde(skip)]
    pub work_dir_name: String,
    pub printer_bin: String,
    pub printer_name: String,
    pub printer_settings: String,
    pub ws_url: String,
    pub ws_context_id: String,
    pub ws_context_name: String,
    pub ws_context_pwd: String,
    pub ws_printer_name: String,
}

impl Display for AppConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AppConfig{{\n  config_file_name: {:?},\n  root_path: {:?},\n  work_dir_name: {:?},\n  printer_bin: {:?},\n  printer_name: {:?},\n  printer_settings: {:?},\n  ws_url: {:?},\n  ws_context_id: {:?},\n  ws_context_name: {:?},\n  ws_context_pwd: {:?},\n  ws_printer_name: {:?}\n}}", self.config_file_name, self.root_path, self.work_dir_name, self.printer_bin, self.printer_name, self.printer_settings, self.ws_url, self.ws_context_id, self.ws_context_name, self.ws_context_pwd, self.ws_printer_name)
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            config_file_name: "cloudprint.config".to_string(),
            root_path: ".".to_string(),
            work_dir_name: "wdir".to_string(),
            printer_bin: "echo".to_string(),
            printer_name: "".to_string(),
            printer_settings: "".to_string(),
            ws_url: String::new(),
            ws_context_id: String::new(),
            ws_context_name: String::new(),
            ws_context_pwd: String::new(),
            ws_printer_name: String::new(),
        }
    }
}

impl AppConfig {
    pub fn generate_config_file() {
        let default_config = AppConfig::default();

        let yaml_string = serde_yaml::to_string(&default_config).unwrap();
        let json_string = serde_json::to_string_pretty(&default_config).unwrap();
        let filename = default_config.config_file_name;

        if let Err(e) = std::fs::write(format!("{}.yaml", filename), yaml_string) {
            log::error!("Error generating .yaml config: {}", e);
        }

        if let Err(e) = std::fs::write(format!("{}.json", filename), json_string) {
            log::error!("Error generating .json config: {}", e);
        }
    }

    pub fn from_config_file() -> AppConfig {
        let default_config = AppConfig::default();

        let mut extensions = std::collections::HashMap::new();
        extensions.insert(FileSupported::JSON, "json");
        extensions.insert(FileSupported::YAML, "yaml");
        extensions.insert(FileSupported::YML, "yml");

        let mut config_file: Option<AppConfig> = None;

        for extension_name in [FileSupported::JSON, FileSupported::YML, FileSupported::YAML] {
            let extension_value = extensions.get(&extension_name).unwrap();
            let file_path = path::Path::new(".").join(format!(
                "{}.{}",
                default_config.config_file_name, extension_value
            ));
            println!("{:?}", file_path);

            let read_file_result = std::fs::read_to_string(file_path);
            let string_input = match read_file_result {
                Err(e) => {
                    error!("{}", e);
                    continue;
                }
                Ok(x) => x,
            };

            config_file = match extension_name {
                FileSupported::JSON => {
                    log::info!("Configuration file .{} found", extension_value);
                    log::info!("\n{}", &string_input);
                    let output: Result<AppConfig, serde_json::Error> =
                        serde_json::from_str(string_input.as_str());
                    match output {
                        Ok(x) => Some(x),
                        Err(e) => {
                            log::error!("Error importing .json file. Input:\n{}", e);
                            panic!();
                        }
                    }
                }
                FileSupported::YAML | FileSupported::YML => {
                    log::info!("Configuration file .{} found", extension_value);
                    log::info!("\n{}", string_input);
                    let output: Result<AppConfig, serde_yaml::Error> =
                        serde_yaml::from_str(string_input.as_str());
                    match output {
                        Ok(x) => Some(x),
                        Err(e) => {
                            log::error!("Error importing .json file. Input:\n{}", e);
                            panic!();
                        }
                    }
                }
            };
            break;
        }

        let output = match config_file {
            Some(mut v) => {
                v.root_path = Self::default().root_path;
                v.config_file_name = Self::default().config_file_name;
                v.work_dir_name = Self::default().work_dir_name;
                v
            }
            None => {
                error!("Config file (print.config.(js|yml|yaml)) does not exist and must be created before initialize application");
                panic!()
            }
        };

        log::info!("App Configuration loaded \n{:}", &output);
        output
    }
}
