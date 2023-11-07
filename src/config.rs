use std::{fs, path};

#[derive(PartialEq, Eq, Hash)]
pub enum FileSupported {
    JSON,
    YML,
    YAML,
}

use log::{info, warn, error};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfigFromFile {
    pub root_path: Option<String>,
    pub work_dir_name: Option<String>,
    pub printer_bin: Option<String>,
    pub printer_args: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    config_file_name: String,
    pub root_path: String,
    pub work_dir_name: String,
    pub printer_bin: String,
    pub printer_args: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            config_file_name: "printclient.config".to_string(),
            root_path: ".".to_string(),
            work_dir_name: "wdir".to_string(),
            printer_bin: "cat".to_string(),
            printer_args: vec![]
        }
    }
}

impl AppConfig {
    // pub fn generate_config_file() {
    //     let default_config = AppConfig::default();

    //     let yaml_string = serde_yaml::to_string(&default_config).unwrap();
    //     let json_string = serde_json::to_string_pretty(&default_config).unwrap();
    //     let filename = default_config.config_file_name;

    //     if let Err(e) = std::fs::write(format!("{}.yaml", filename), yaml_string) {
    //         log::error!("Erro ao gerar yaml config: {}", e);
    //     }

    //     if let Err(e) = std::fs::write(format!("{}.json", filename), json_string) {
    //         log::error!("Erro ao gerar json config: {}", e);
    //     }
    // }

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
                    log::info!("Arquivo de configuracao .{} encontrado", extension_value);
                    log::info!("\n{}", &string_input);
                    let output: Result<AppConfigFromFile, serde_json::Error> =
                        serde_json::from_str(string_input.as_str());
                    match output {
                        Ok(x) => Some(x),
                        Err(e) => {
                            log::error!("Erro ao importar arquivo json");
                            log::error!("{}", e);
                            panic!();
                        }
                    }
                }
                FileSupported::YAML | FileSupported::YML => {
                    log::info!("Arquivo de configuracao .{} encontrado", extension_value);
                    log::info!("\n{}", string_input);
                    let output: Result<AppConfigFromFile, serde_yaml::Error> =
                        serde_yaml::from_str(string_input.as_str());
                    match output {
                        Ok(x) => Some(x),
                        Err(e) => {
                            log::error!("erro ao importar arquivo yml");
                            log::error!("{}", e);
                            panic!();
                        }
                    }
                }
            };
            break;
        }

        let output = match config_file {
            Some(v) => AppConfig {
                root_path: v.root_path.or(Some(default_config.root_path)).unwrap(),
                printer_bin: v.printer_bin.or(Some(default_config.printer_bin)).unwrap(),
                work_dir_name: v
                    .work_dir_name
                    .or(Some(default_config.work_dir_name))
                    .unwrap(),
                ..Default::default()
            },
            None => default_config,
        };

        log::info!("Configuração carregada \n{:?}", &output);
        output
    }

    pub fn generate_working_dir(config: &AppConfig) {
        let pending_path = path::Path::new(config.root_path.as_str())
            .join(config.work_dir_name.as_str())
            .join("pending");
        let error_path = path::Path::new(config.root_path.as_str())
            .join(config.work_dir_name.as_str())
            .join("error");
        let ok_path = path::Path::new(config.root_path.as_str())
            .join(config.work_dir_name.as_str())
            .join("ok");
        for path in [pending_path, error_path, ok_path] {
            let path_str = path.to_string_lossy();
            info!("Criando \"{}\"", path_str);
            match fs::create_dir_all(path.as_path()) {
                Ok(()) => {
                    info!("\"{}\" adicionado", path_str);
                }
                Err(_e) => {
                    warn!("Erro ao criar diretório \"{}\"", path_str);
                }
            }
        }
    }
}
