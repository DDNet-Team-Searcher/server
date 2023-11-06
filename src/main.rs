mod handlers;
mod protos;
mod state;

use anyhow::Result;
use handlers::{shutdown::shutdown_server, start::start_server};
use protobuf::Message;
use protos::{
    request::{Action, Request},
    response::Response,
};
use state::State;
use std::{collections::HashMap, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

const ALLOWED_IPS: &[&'static str] = &["127.0.0.1"];

//HOLY FUCKING MOLLY, this is fine... dont look at all these arc mutexes... please
async fn proccess(
    socket: Arc<Mutex<TcpStream>>,
    state: Arc<Mutex<State>>,
    connected_sockets: Arc<Mutex<HashMap<String, Arc<Mutex<TcpStream>>>>>,
) {
    loop {
        let mut guard = socket.lock().await;
        let mut reader = BufReader::new(&mut *guard);
        let mut buf = [0; 1024];

        let n = reader.read(&mut buf).await.unwrap();

        if n == 0 {
            //disconnected
            connected_sockets
                .lock()
                .await
                .remove(&guard.peer_addr().unwrap().to_string());

            break;
        }

        //dont get dead locked :kekw:
        drop(guard);

        let request = match Request::parse_from_bytes(&buf[0..n]) {
            Ok(req) => req,
            Err(_) => {
                println!("Stupid bitch cant send right data");
                break;
            }
        };

        let response: Response;

        match request.action.unwrap() {
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

        let guard = connected_sockets.lock().await;

        for sock in guard.values().into_iter() {
            let mut socket_guard = sock.lock().await;

            socket_guard
                .write_all(&response.write_to_bytes().unwrap())
                .await
                .unwrap();
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    //TODO: add logging
    let listener = TcpListener::bind("127.0.0.1:3030").await?;
    let state = Arc::new(Mutex::new(State::new()));
    let connected_sockets: Arc<Mutex<HashMap<String, Arc<Mutex<TcpStream>>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (socket, addr) = listener.accept().await?;
        let socket_arc = Arc::new(Mutex::new(socket));

        if !ALLOWED_IPS.contains(&&addr.ip().to_string()[..]) {
            continue;
        }

        {
            connected_sockets
                .lock()
                .await
                .insert(addr.to_string(), Arc::clone(&socket_arc));
        }

        let state_arc = Arc::clone(&state);
        let connected_sockets_arc = Arc::clone(&connected_sockets);

        tokio::spawn(async move {
            proccess(socket_arc, state_arc, connected_sockets_arc).await;
        });
    }
}
