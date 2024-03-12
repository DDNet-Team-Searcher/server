use crate::{
    protos::response::{response::ResponseCode, Response, Start},
    state::State,
};
use protobuf::EnumOrUnknown;
use rand::distributions::Alphanumeric;
use rand::prelude::*;
use std::{ops::Range, process::Stdio, sync::Arc};
use tokio::{process::Command, sync::Mutex};

const PORTS_RANGE: Range<usize> = 2000..3000;

fn gimme_port(state: &State, rng: &mut ThreadRng) -> usize {
    let rand = rng.gen_range(PORTS_RANGE);

    if state.is_port_taken(rand) {
        return gimme_port(state, rng);
    }

    return rand;
}

fn generate_password(rng: &mut ThreadRng) -> String {
    return rng
        .sample_iter(&Alphanumeric)
        .take(20)
        .map(char::from)
        .collect();
}

#[tracing::instrument(name = "start server", skip(state, happening_id, map_name))]
pub async fn start_server(
    state: Arc<Mutex<State>>,
    happening_id: usize,
    map_name: String,
) -> Response {
    let mut guard = state.lock().await;
    let mut rng = rand::thread_rng();

    let port = gimme_port(&guard, &mut rng);
    let password = generate_password(&mut rng);
    let id = guard.add_server(happening_id, port).unwrap();

    let server_args = format!(
        "sv_id {}; sv_happening_id {}; sv_shutdown_after_finish 1; sv_port {}; password {}; sv_map {}",
        id, happening_id, port, password, map_name,
    );

    match Command::new(
        "./".to_owned()
            + &std::env::var("DDNET_EXECUTABLE_NAME").expect("DDNET_EXECUTABLE_NAME has to be set"),
    )
    .current_dir(
        std::env::var("DDNET_EXECUTABLE_PATH").expect("DDNET_EXECUTABLE_PATH has to be set"),
    )
    .arg(server_args)
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .spawn()
    {
        Ok(_) => {}
        Err(err) => {
            tracing::error!("failed to spawn ddnet server process: {:?}", err);

            let mut response = Response::new();
            response.code = EnumOrUnknown::from(ResponseCode::WHOOPSIE_DAISY);

            return response;
        }
    };

    tracing::info!(
        "started game server on port {} with password {} and map {}",
        port,
        password,
        map_name
    );

    let mut response_start = Start::new();
    response_start.password = password;
    response_start.port = port as u32;

    let mut response = Response::new();
    response.code = EnumOrUnknown::from(ResponseCode::OK);
    response.set_start(response_start);

    return response;
}
