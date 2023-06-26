use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::errors::Errors;

#[derive(Serialize, Deserialize, Debug)]
pub struct Shutdown {
    pid: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShutdownSuccess {
    status: String,
    pid: u32,
}

impl Shutdown {
    pub async fn shutdown(&self) -> Result<ShutdownSuccess, Errors> {
        match Command::new("kill")
            .args(["-9", &self.pid.to_string()])
            .output()
            .await
        {
            Ok(_) => (),
            Err(_) => return Err(Errors::Unknown),
        };

        Ok(ShutdownSuccess {
            status: "SERVER_SHUTDOWN_SUCCESSFULLY".to_string(),
            pid: self.pid,
        })
    }
}
