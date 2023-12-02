mod handlers;
mod ipc;
mod protos;
mod state;

use anyhow::Result;
use dotenv::dotenv;
use handlers::{info::get_info, shutdown::shutdown_server, start::start_server};
use protobuf::Message;
use protos::{
    request::{Action, Request},
    response::Response,
};
use state::State;
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::{mpsc, Mutex},
};

const ALLOWED_IPS: &[&'static str] = &["127.0.0.1"];
const MAX_SERVERS: usize = 10;

async fn proccess(mut socket: TcpStream, state: Arc<Mutex<State>>, addr: SocketAddr) {
    let (tx, mut rx) = mpsc::channel::<Vec<u8>>(512);

    {
        state.lock().await.add_peer(addr, tx);
    }
    println!("Connected");

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

                    let response: Response;

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
                    };

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
    let listener = TcpListener::bind("127.0.0.1:42069").await?;
    let state = Arc::new(Mutex::new(State::new(MAX_SERVERS)));

    loop {
        let (socket, addr) = listener.accept().await?;

        if !ALLOWED_IPS.contains(&&addr.ip().to_string()[..]) {
            continue;
        }

        let state = Arc::clone(&state);

        tokio::spawn(async move {
            proccess(socket, state.clone(), addr).await;
        });
    }
}
