use crate::{
    protos::response::{Response, ResponseCode},
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

pub async fn start_server(state: Arc<Mutex<State>>, id: usize, map_name: String) -> Response {
    let mut guard = state.lock().await;
    let mut rng = rand::thread_rng();

    let port = gimme_port(&guard, &mut rng);
    let password = generate_password(&mut rng);

    let server_args = format!(
        "sv_port {}; password {}; sv_map {}",
        port, password, map_name,
    );

    let child = match Command::new("./DDNet-Server")
        .current_dir("../DDnet-Team-Searcher-Server/build")
        .arg(server_args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(err) => {
            dbg!(err);

            let mut response = Response::new();
            response.response_code = EnumOrUnknown::from(ResponseCode::WHOOPSIE_DAISY);

            return response;
        }
    };

    guard.add_server(id, child.id().unwrap() as usize).unwrap();

    println!(
        "Started a server on port {} with password {}",
        port, &password
    );

    let mut res = Response::new();

    res.password = Some(password);
    res.port = Some(port as u32);
    res.response_code = EnumOrUnknown::from(ResponseCode::OK);

    return res;
}
