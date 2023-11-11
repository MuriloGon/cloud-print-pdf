use std::path;

#[derive(PartialEq, Eq, Hash)]
pub enum FileSupported {
    JSON,
    YML,
    YAML,
}

use log::error;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfigFromFile {
    pub root_path: Option<String>,
    pub work_dir_name: Option<String>,
    pub printer_bin: Option<String>,
    pub printer_args: Option<Vec<String>>,
    pub ws_url: Option<String>,
    pub ws_context_id: Option<String>,
    pub ws_context_name: Option<String>,
    pub ws_context_pwd: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    config_file_name: String,
    pub root_path: String,
    pub work_dir_name: String,
    pub printer_bin: String,
    pub printer_args: Vec<String>,
    pub ws_url: String,
    pub ws_context_id: String,
    pub ws_context_name: String,
    pub ws_context_pwd: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            config_file_name: "printclient.config".to_string(),
            root_path: ".".to_string(),
            work_dir_name: "wdir".to_string(),
            printer_bin: "cat".to_string(),
            printer_args: vec![],
            ws_url: String::new(),
            ws_context_id: String::new(),
            ws_context_name: String::new(),
            ws_context_pwd: String::new(),
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

        let mut config_file: Option<AppConfigFromFile> = None;

        for extension_name in [FileSupported::JSON, FileSupported::YML, FileSupported::YAML] {
            let extension_value = extensions.get(&extension_name).unwrap();
            let file_path = path::Path::new(".").join(format!(
                "{}.{}",
                default_config.config_file_name, extension_value
            ));

            let read_file_result = std::fs::read_to_string(file_path);
            let string_input = match read_file_result {
                Err(e) => {
                    error!("{}", e);
                    panic!();
                }
                Ok(x) => x,
            };

            config_file = match extension_name {
                FileSupported::JSON => {
                    log::info!("Configuration file .{} found", extension_value);
                    log::info!("\n{}", &string_input);
                    let output: Result<AppConfigFromFile, serde_json::Error> =
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
                    let output: Result<AppConfigFromFile, serde_yaml::Error> =
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
            Some(v) => AppConfig {
                config_file_name: Self::default().config_file_name,
                root_path: v.root_path.or(Some(default_config.root_path)).unwrap(),
                printer_bin: v.printer_bin.or(Some(default_config.printer_bin)).unwrap(),
                work_dir_name: v
                    .work_dir_name
                    .or(Some(default_config.work_dir_name))
                    .unwrap(),
                ws_url: v.ws_url.or(Some(default_config.ws_url)).unwrap(),
                printer_args: v
                    .printer_args
                    .or(Some(default_config.printer_args))
                    .unwrap(),
                ws_context_id: v
                    .ws_context_id
                    .or(Some(default_config.ws_context_id))
                    .unwrap(),
                ws_context_name: v
                    .ws_context_name
                    .or(Some(default_config.ws_context_name))
                    .unwrap(),
                ws_context_pwd: v
                    .ws_context_pwd
                    .or(Some(default_config.ws_context_pwd))
                    .unwrap(),
            },
            None => default_config,
        };

        log::info!("App Configuration loaded \n{:?}", &output);
        output
    }
}
