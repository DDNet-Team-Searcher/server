use crate::errors::Errors;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::process::Command;

#[derive(Serialize, Deserialize, Debug)]
pub struct Start {
    map_name: String,
    password: String,
    config_file: String,
    id: u32,
    port: u16,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StartSuccess {
    status: String,
    pid: u32,
    id: u32,
    password: String,
    port: u16,
}

impl Start {
    pub fn start(&self) -> Result<StartSuccess, Errors> {
        let server_string = format!(
            "sv_port {}; password {}; sv_map {}",
            self.port, self.password, self.map_name,
        );

        let child = match Command::new("./DDnet-Server")
            .current_dir("../ddnet-server")
            .args([&server_string, "-f", &self.config_file])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(child) => child,
            Err(_) => return Err(Errors::Unknown),
        };

        Ok(StartSuccess {
            id: self.id,
            pid: child.id().unwrap(),
            status: "SERVER_STARTED_SUCCESSFULLY".to_string(),
            password: self.password.clone(),
            port: self.port,
        })
    }
}
