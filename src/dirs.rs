use std::{fs, path};

use log::{info, warn};

use crate::config::AppConfig;

#[derive(Clone)]
pub struct Dirs;

impl Dirs {
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
