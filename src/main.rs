mod config;
mod handlers;
mod ipc;
mod protos;
mod state;

use crate::protos::request::request::Action;
use anyhow::Result;
use config::get_config;
use dotenv::dotenv;
use handlers::{info::get_info, shutdown::shutdown_server, start::start_server};
use protobuf::Message;
use protos::{request::Request, response::Response};
use state::State;
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::{mpsc, Mutex},
};

async fn proccess(mut socket: TcpStream, state: Arc<Mutex<State>>, addr: SocketAddr) {
    let (tx, mut rx) = mpsc::channel::<Vec<u8>>(512);

    {
        state.lock().await.add_peer(addr, tx);
    }

    loop {
        let mut reader = BufReader::new(&mut socket);
        let mut buf = [0; 1024];

        tokio::select! {
            Some(msg) = rx.recv() => {
                socket.write_all(&msg).await.unwrap();
            }
            result = reader.read(&mut buf) => match result {
                Ok(size) => {
                    if size == 0 {
                        break;
                    }

                    let request = match Request::parse_from_bytes(&buf[0..size]) {
                        Ok(req) => req,
                        Err(_) => {
                            println!("Stupid bitch cant send right data");
                            break;
                        }
                    };

                    let mut response: Response;

                    match request.action.unwrap() {
                        Action::INFO => {
                            response = get_info(Arc::clone(&state)).await;
                        }
                        Action::START => {
                            response = start_server(
                                Arc::clone(&state),
                                request.id as usize,
                                request.map_name.unwrap(),
                            )
                            .await;
                        }
                        Action::SHUTDOWN => {
                            response = shutdown_server(Arc::clone(&state), request.id as usize).await;
                        }
                        Action::UNKNOWN => {
                            unreachable!();
                        }
                    };

                    response.origin = request.origin;

                    state.lock().await.broadcast_msg(response.write_to_bytes().unwrap().to_vec()).await;
                }
                Err(err) => {
                    dbg!(err);
                    eprintln!("We're fucked");
                }
            }
        }
    }

    state.lock().await.remove_peer(&addr);
    println!("User disconnected");
}

#[tokio::main]
async fn main() -> Result<()> {
    //TODO: add logging
    dotenv().ok();
    let config = get_config().expect("Failed to get config");
    let listener = TcpListener::bind(format!(
        "{}:{}",
        config.application.host, config.application.port
    ))
    .await?;
    let state = Arc::new(Mutex::new(State::new(
        config.application.max_servers as usize,
    )));

    loop {
        let (socket, addr) = listener.accept().await?;

        if !&config
            .application
            .allowed_ips
            .contains(&addr.ip().to_string())
        {
            continue;
        }

        let state = Arc::clone(&state);

        tokio::spawn(async move {
            proccess(socket, state.clone(), addr).await;
        });
    }
}
