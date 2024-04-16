use crate::{
    config::Config,
    handlers::{get_info, shutdown_server, start_server, stats},
    protos::{
        request::{request::Action, Request},
        response::Response,
    },
    state::State,
};
use protobuf::Message;
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::{mpsc, Mutex},
};

pub struct Application {
    listener: TcpListener,
    state: Arc<Mutex<State>>,
    allowed_ips: Vec<String>,
}

impl Application {
    pub async fn new(config: Config) -> Result<Self, std::io::Error> {
        let listener = TcpListener::bind(format!(
            "{}:{}",
            config.application.host, config.application.port
        ))
        .await?;
        let state = Arc::new(Mutex::new(State::new(
            config.application.max_servers as u32,
        )));

        Ok(Self {
            listener,
            state,
            allowed_ips: config.application.allowed_ips,
        })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        let app = Arc::new(self);

        loop {
            let app = Arc::clone(&app);
            let (socket, addr) = app.listener.accept().await?;

            if !app.allowed_ips.is_empty() && !app.allowed_ips.contains(&&addr.ip().to_string()) {
                tracing::info!("ip {} wasn't in white list, skipping...", addr);
                continue;
            }

            tokio::spawn(async move {
                app.proccess(socket, addr).await;
            });
        }
    }

    #[tracing::instrument(name = "process incoming request", skip_all)]
    async fn proccess(&self, mut socket: TcpStream, addr: SocketAddr) {
        let (tx, mut rx) = mpsc::channel::<Vec<u8>>(512);

        {
            self.state.lock().await.add_peer(addr, tx);
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
                                response = get_info(Arc::clone(&self.state)).await;
                            }
                            Action::START => {
                                response = start_server(
                                    Arc::clone(&self.state),
                                    request.id as usize,
                                    request.map_name,
                                )
                                .await;
                            }
                            Action::SHUTDOWN => {
                                response = shutdown_server(Arc::clone(&self.state), request.id as usize).await;
                            }
                            Action::STATS=> {
                                response = stats(Arc::clone(&self.state)).await;
                            }
                            Action::UNKNOWN => {
                                tracing::debug_span!("got request with unkown action", ?request);
                                break;
                            }
                        };

                        response.origin = request.origin;

                        self.state.lock().await.broadcast_msg(response.write_to_bytes().unwrap().to_vec()).await;
                    }
                    Err(err) => {
                        tracing::error!("error occurred while processing user request: {:?}", err);
                        break;
                    }
                }
            }
        }

        self.state.lock().await.remove_peer(&addr);
    }
}
