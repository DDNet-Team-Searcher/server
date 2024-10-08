use crate::{protos::response::Response, state::State};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tracing::instrument("get info", skip_all)]
pub async fn get_info(state: Arc<Mutex<State>>) -> Response {
    state.lock().await.get_info().into()
}
