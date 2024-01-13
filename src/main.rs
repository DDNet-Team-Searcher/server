mod config;
mod handlers;
mod ipc;
mod protos;
mod state;
mod telemetry;

use crate::protos::request::request::Action;
use anyhow::Result;
use config::get_config;
use dotenv::dotenv;
use handlers::{get_info, shutdown_server, start_server};
use protobuf::Message;
use protos::{request::Request, response::Response};
use state::State;
use std::{net::SocketAddr, sync::Arc};
use telemetry::{get_subscriber, init_subscriber};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::{mpsc, Mutex},
};

#[tracing::instrument(name = "process incoming request", skip(socket, state, addr))]
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
                            tracing::debug_span!("coudn't parse incoming request");
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
                            tracing::debug_span!("got request with unkown action", ?request);
                            break;
                        }
                    };

                    response.origin = request.origin;

                    state.lock().await.broadcast_msg(response.write_to_bytes().unwrap().to_vec()).await;
                }
                Err(err) => {
                    tracing::error!("error occurred while processing user request: {:?}", err);
                    break;
                }
            }
        }
    }

    state.lock().await.remove_peer(&addr);
}

#[tokio::main]
async fn main() -> Result<()> {
    let default_filter_level = "debug".to_string();
    let subscriber_name = "ddts".to_string();

    dotenv().ok();
    init_subscriber(get_subscriber(
        subscriber_name,
        default_filter_level,
        std::io::stdout,
    ));
    let config = get_config().expect("failed to get config");
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

        if !config.application.allowed_ips.is_empty()
            && !&config
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
