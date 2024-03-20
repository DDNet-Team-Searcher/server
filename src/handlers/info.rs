use crate::{protos::response::Response, state::State};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tracing::instrument("get info")]
pub async fn get_info(state: Arc<Mutex<State>>) -> Response {
    let info = state.lock().await.get_info();

    return info.into();
}
