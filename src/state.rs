use crate::ipc::SharedMemory;
use anyhow::Result;
use std::{collections::HashMap, net::SocketAddr};
use tokio::sync::mpsc;

type Tx = mpsc::Sender<Vec<u8>>;
type Rx = mpsc::Receiver<Vec<u8>>;

pub struct Info {
    pub used: usize,
    pub max: usize,
}

#[derive(Debug, Clone, Copy)]
struct Server {
    port: usize,
    happening_id: usize,
}

#[derive(Debug)]
pub struct State {
    pub shared_memory: SharedMemory,
    peers: HashMap<SocketAddr, Tx>,
    servers: Vec<Option<Server>>,
    max: usize,
    in_use: usize,
}

impl State {
    pub fn new(max: usize) -> Self {
        let mut servers = Vec::with_capacity(max);
        servers.resize(max, None);

        return Self {
            servers,
            shared_memory: SharedMemory::new(max),
            peers: HashMap::new(),
            max,
            in_use: 0,
        };
    }

    pub async fn broadcast_msg(&mut self, msg: Vec<u8>) {
        for peer in self.peers.iter_mut() {
            peer.1.send(msg.clone()).await.unwrap();
        }
    }

    pub fn add_peer(&mut self, socket_addr: SocketAddr, tx: Tx) {
        self.peers.insert(socket_addr, tx);
    }

    pub fn remove_peer(&mut self, socket_addr: &SocketAddr) {
        self.peers.remove(socket_addr);
    }

    pub fn add_server(&mut self, happening_id: usize, port: usize) -> Result<usize> {
        //TODO: add checks...

        let mut id = 0;

        for i in 0..self.servers.len() {
            if self.servers[i].is_none() {
                id = i;
                break;
            }
        }

        self.servers[id] = Some(Server { happening_id, port });
        self.in_use += 1;

        return Ok(id);
    }

    pub fn remove_server(&mut self, happening_id: usize) -> Option<usize> {
        for i in 0..self.servers.len() {
            if let Some(server) = self.servers[i] {
                if server.happening_id == happening_id {
                    self.servers[i] = None;
                    self.in_use -= 1;

                    return Some(i);
                }
            }
        }

        return None;
    }

    pub fn get_info(&self) -> Info {
        return Info {
            used: self.in_use,
            max: self.max,
        };
    }

    pub fn is_port_taken(&self, port: usize) -> bool {
        for server in &self.servers {
            if let Some(srv) = server {
                if srv.port == port {
                    return true;
                }
            }
        }

        return false;
    }
}
