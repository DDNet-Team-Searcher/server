mod errors;
mod shutdown;
mod start;

use serde::{Deserialize, Serialize};
use serde_json;
use shutdown::Shutdown;
use start::Start;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};

use crate::errors::Errors;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum Request {
    StartRequest(Start),
    ShutdownRequest(Shutdown),
}

async fn proccess(mut socket: TcpStream) {
    loop {
        let mut buf_reader = BufReader::new(&mut socket);
        let mut line = String::new();
        buf_reader.read_line(&mut line).await.unwrap();

        let request: serde_json::Result<Request> = serde_json::from_str(&line);

        match request {
            Ok(data) => {
                let res: String;

                match data {
                    Request::StartRequest(start) => match start.start() {
                        Ok(data) => res = serde_json::to_string(&data).unwrap(),
                        Err(err) => res = err.to_string(),
                    },
                    Request::ShutdownRequest(shutdown) => match shutdown.shutdown().await {
                        Ok(data) => res = serde_json::to_string(&data).unwrap(),
                        Err(err) => res = err.to_string(),
                    },
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

        // connected
        proccess(socket).await;
    }
}
