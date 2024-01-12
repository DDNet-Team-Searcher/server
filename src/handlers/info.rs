use crate::{
    protos::response::{response::ResponseCode, Response},
    state::State,
};
use protobuf::EnumOrUnknown;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn get_info(state: Arc<Mutex<State>>) -> Response {
    let info = state.lock().await.get_info();
    let mut response = Response::new();

    response.response_code = EnumOrUnknown::from(ResponseCode::OK);
    response.used = Some(info.used as u32);
    response.max = Some(info.max as u32);

    return response;
}
