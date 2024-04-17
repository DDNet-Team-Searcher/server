use crate::{
    protos::response::response::ResponseCode,
    protos::response::{Response, Shutdown},
    state::State,
};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tracing::instrument(name = "shutdown server", skip(state))]
pub async fn shutdown_server(state: Arc<Mutex<State>>, happening_id: usize) -> Response {
    let mut state = state.lock().await;
    let id = state.remove_server(happening_id).unwrap();
    state.shared_memory.shutdown_server(id as u32);

    tracing::info!("closed server with id {}", id);

    let mut response_shutdown = Shutdown::new();
    response_shutdown.id = happening_id as u32;

    let mut response = Response::new();
    response.code = ResponseCode::OK.into();
    response.set_shutdown(response_shutdown);

    return response;
}
