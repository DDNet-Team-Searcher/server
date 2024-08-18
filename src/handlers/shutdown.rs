use crate::{
    protos::response::{
        response::{Data, ResponseCode},
        Response, Shutdown,
    },
    state::State,
};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tracing::instrument(name = "shutdown server", skip(state))]
pub async fn shutdown_server(state: Arc<Mutex<State>>, happening_id: usize) -> Response {
    let mut state = state.lock().await;
    let mut server = state.remove_server(happening_id).unwrap();

    server.shutdown().await;
    tracing::info!("closed server with happeing id {}", server.happening_id);

    let data = Shutdown {
        id: happening_id as u32,
        ..Default::default()
    };

    Response {
        code: ResponseCode::OK.into(),
        data: Some(Data::Shutdown(data)),
        ..Default::default()
    }
}
