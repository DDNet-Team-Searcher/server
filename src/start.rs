use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Start {
    pub action: String,
    pub map_name: String,
    pub id: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StartSuccess {
    pub status: String,
    pub password: String,
    pub port: u32,
}
