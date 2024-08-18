use crate::{protos::response::Response, state::State};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tracing::instrument("get server stats", skip_all)]
pub async fn stats(state: Arc<Mutex<State>>) -> Response {
    state.lock().await.stats().into()
}
