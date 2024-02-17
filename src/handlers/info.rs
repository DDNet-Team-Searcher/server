use crate::{
    protos::response::{response::ResponseCode, Response},
    state::State,
};
use protobuf::EnumOrUnknown;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tracing::instrument("get info")]
pub async fn get_info(state: Arc<Mutex<State>>) -> Response {
    let info = state.lock().await.get_info();

    let mut response_info = crate::protos::response::Info::new();
    response_info.used = Some(info.used as u32);
    response_info.max = Some(info.max as u32);

    let mut response = Response::new();
    response.code = EnumOrUnknown::from(ResponseCode::OK);
    response.set_info(response_info);

    return response;
}
