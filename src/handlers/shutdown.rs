use crate::{protos::response::Response, state::State};
use protobuf::EnumOrUnknown;
use std::sync::Arc;
use tokio::{process::Command, sync::Mutex};

use crate::protos::response::ResponseCode;

pub async fn shutdown_server(state: Arc<Mutex<State>>, id: usize) -> Response {
    let mut guard = state.lock().await;
    let pid = guard.remove_server(id);

    if pid == None {
        let mut response = Response::new();
        response.response_code = EnumOrUnknown::from(ResponseCode::NOT_FOUND);

        return response;
    }

    //500iq, no need applause
    let result = Command::new("kill")
        .args(["-9", &pid.unwrap().to_string()])
        .output()
        .await;

    if let Err(_) = result {
        let mut response = Response::new();
        response.response_code = EnumOrUnknown::from(ResponseCode::WHOOPSIE_DAISY);

        return response;
    }

    println!("Closed server with pid {}", pid.unwrap());

    let mut response = Response::new();
    response.response_code = EnumOrUnknown::from(ResponseCode::OK);

    return response;
}
