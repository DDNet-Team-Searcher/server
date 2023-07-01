use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Shutdown {
    pub action: String,
    pub id: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShutdownSuccess {
    pub status: String,
}
