use crate::{protos::response::response::ResponseCode, protos::response::Response, state::State};
use protobuf::EnumOrUnknown;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn shutdown_server(state: Arc<Mutex<State>>, happening_id: usize) -> Response {
    let mut state = state.lock().await;
    let id = state.remove_server(happening_id).unwrap();
    state.shared_memory.shutdown_server(id as u32);

    println!("Closed server with id {}", id);

    let mut response = Response::new();
    response.response_code = EnumOrUnknown::from(ResponseCode::OK);
    response.id = Some(happening_id as u32);

    return response;
}
