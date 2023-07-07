use rand::distributions::{Alphanumeric, DistString};
use std::{collections::HashMap, process::Stdio};

use rand::Rng;
use tokio::process::Command;

use crate::errors::Errors;

fn random_port() -> u32 {
    //FIXME: what if it will generate the same number again
    rand::thread_rng().gen_range(1000..=9999)
}

fn generate_password() -> String {
    Alphanumeric.sample_string(&mut rand::thread_rng(), 20)
}

#[derive(Debug)]
pub struct GameServersHandler {
    servers: HashMap<usize, u32>,
}

impl GameServersHandler {
    pub fn new() -> Self {
        generate_password();
        Self {
            servers: HashMap::new(),
        }
    }

    pub fn start(&mut self, id: usize, map_name: &str) -> Result<(u32, String), Errors> {
        let port = random_port();
        let password = generate_password();

        let server_string = format!(
            "sv_port {}; password {}; sv_map {}",
            port, password, map_name,
        );

        let child = match Command::new("./DDnet-Server")
            .current_dir("../ddnet-server")
            .arg(server_string)
            // .args([&server_string, "-f", &self.config_file])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(child) => child,
            Err(_) => return Err(Errors::Unknown),
        };

        self.servers.insert(id, child.id().unwrap());

        println!(
            "Started server on port: {}, with password: {}",
            port, password
        );

        Ok((port, password))
    }

    pub async fn shutdown(&mut self, id: usize) -> Result<(), Errors> {
        let pid = self.servers.remove(&id).unwrap(); // FIXME: thread 'tokio-runtime-worker' panicked at 'called `Option::unwrap()` on a `None` value'

        match Command::new("kill")
            .args(["-9", &pid.to_string()])
            .output()
            .await
        {
            Ok(_) => {
                println!("Killed server with id: {}", id);

                return Ok(());
            }
            Err(_) => return Err(Errors::Unknown),
        };
    }
}
