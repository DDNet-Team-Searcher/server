mod errors;
mod servers_handler;
mod shutdown;
mod start;

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json;
use shutdown::{Shutdown, ShutdownSuccess};
use start::{Start, StartSuccess};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

use crate::{errors::Errors, servers_handler::GameServersHandler};

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum Request {
    StartRequest(Start),
    ShutdownRequest(Shutdown),
}

async fn proccess(mut socket: TcpStream, game_servers_handler: Arc<Mutex<GameServersHandler>>) {
    loop {
        let mut buf_reader = BufReader::new(&mut socket);
        let mut line = String::new();

        buf_reader.read_line(&mut line).await.unwrap();

        let request: serde_json::Result<Request> = serde_json::from_str(&line);
        let mut da_thing = game_servers_handler.lock().await;

        match request {
            Ok(data) => {
                let mut res: String = String::new();

                match data {
                    Request::StartRequest(data) => {
                        if data.action == "START" {
                            match da_thing.start(data.id, &data.map_name) {
                                Ok((port, password)) => {
                                    res = serde_json::to_string(&StartSuccess {
                                        status: "SERVER_STARTED_SUCCESSFULLY".to_string(),
                                        port,
                                        password,
                                    })
                                    .unwrap();
                                }
                                Err(err) => {
                                    res = err.to_string();
                                }
                            }
                        }
                    }
                    Request::ShutdownRequest(data) => {
                        if data.action == "SHUTDOWN" {
                            match da_thing.shutdown(data.id).await {
                                Ok(_) => {
                                    res = serde_json::to_string(&ShutdownSuccess {
                                        status: "SERVER_SHUTDOWN_SUCCESSFULLY".to_string(),
                                    })
                                    .unwrap();
                                }
                                Err(err) => {
                                    res = err.to_string();
                                }
                            };
                        }
                    }
                }

                socket.write_all(res.as_bytes()).await.unwrap();
            }
            Err(_) => {
                socket
                    .write_all(Errors::BadData.to_string().as_bytes())
                    .await
                    .unwrap();
            }
        }

        if line.as_bytes().len() == 0 {
            // disconnected
            break;
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let game_servers_handler = Arc::new(Mutex::new(GameServersHandler::new()));
    const ALLOWED_IPS: &[&'static str] = &["192.168.56.1"];

    let listener = TcpListener::bind("192.168.56.1:6942").await?;

    loop {
        let (mut socket, addr) = listener.accept().await?;

        if !ALLOWED_IPS.contains(&&addr.ip().to_string()[..]) {
            socket
                .write_all(Errors::AccessDenied.to_string().as_bytes())
                .await
                .unwrap();
            continue;
        }

        let da_thing = Arc::clone(&game_servers_handler);

        // connected
        tokio::spawn(async {
            proccess(socket, da_thing).await;
        });
    }
}
