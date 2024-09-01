use crate::{
    protos::response::{
        response::{Data, ResponseCode},
        Response, Shutdown, Start,
    },
    state::State,
};
use protobuf::Message;
use rand::distributions::Alphanumeric;
use rand::prelude::*;
use std::{ops::Range, path::PathBuf, process::Stdio, sync::Arc};
use tokio::{process::Command, sync::Mutex};

const PORTS_RANGE: Range<u16> = 2000..3000;

fn gimme_port(state: &State) -> u16 {
    let port = rand::thread_rng().gen_range(PORTS_RANGE);

    if state.is_port_taken(port) {
        gimme_port(state)
    } else {
        port
    }
}

fn generate_password() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(20)
        .map(char::from)
        .collect()
}

#[tracing::instrument(name = "start server", skip(state))]
pub async fn start_server(
    state: Arc<Mutex<State>>,
    happening_id: usize,
    map_name: String,
) -> Response {
    let mut guard = state.lock().await;
    let port = gimme_port(&guard);
    let password = generate_password();
    let id = guard.empty_index().expect("whoopsie daisy fix me pls owo");
    let logfile = format!(
        "./logs/{}_{}.log",
        chrono::Utc::now().to_rfc3339(),
        happening_id
    );
    let fifo = format!("{id}.fifo");
    let ddnet_server_path = PathBuf::from(
        std::env::var("DDNET_EXECUTABLE_PATH").expect("DDNET_EXECUTABLE_PATH has to be set"),
    );
    let server_args = format!("sv_id {id}; sv_happening_id {happening_id}; sv_shutdown_after_finish 1; sv_port {port}; password {password}; sv_map {map_name}; logfile {logfile}; sv_input_fifo {fifo}");

    match Command::new(format!(
        "./{}",
        std::env::var("DDNET_EXECUTABLE_NAME").expect("DDNET_EXECUTABLE_NAME has to be set")
    ))
    .current_dir(&ddnet_server_path)
    .arg(server_args)
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .spawn()
    {
        Ok(mut child) => {
            //NOTE: sleep 500ms to let ddnet server create fifo file
            std::thread::sleep(std::time::Duration::from_millis(500));
            let mut fifo_path = ddnet_server_path;
            fifo_path.push(fifo);

            guard
                .add_server(
                    id,
                    happening_id,
                    port,
                    child.id().expect("failed to get ddnet server pid"),
                    password.clone(),
                    map_name.clone(),
                    &fifo_path,
                )
                .await;
            tracing::info!(
                "started game server on port {} with password {} and map {}",
                port,
                password,
                map_name
            );

            let state = Arc::clone(&state);
            tokio::spawn(async move {
                let code = child.wait().await.unwrap();
                let response = Response {
                    code: ResponseCode::OK.into(),
                    data: Some(Data::Shutdown(Shutdown {
                        id: happening_id as u32,
                        ..Default::default()
                    })),
                    ..Default::default()
                };

                tracing::debug!("Server for happening {happening_id} exited with code {code}");
                state
                    .lock()
                    .await
                    .broadcast_msg(response.write_to_bytes().unwrap())
                    .await;
            });

            let data = Start {
                password,
                port: port as u32,
                ..Default::default()
            };

            Response {
                code: ResponseCode::OK.into(),
                data: Some(Data::Start(data)),
                ..Default::default()
            }
        }
        Err(err) => {
            tracing::error!("failed to spawn ddnet server process: {:?}", err);

            Response {
                code: ResponseCode::WHOOPSIE_DAISY.into(),
                ..Default::default()
            }
        }
    }
}
