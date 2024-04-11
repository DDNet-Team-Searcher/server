use crate::{protos::response::Response, state::State};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tracing::instrument("get server stats", skip(state))]
pub async fn stats(state: Arc<Mutex<State>>) -> Response {
    let stats = state.lock().await.stats();

    return stats.into();
}
