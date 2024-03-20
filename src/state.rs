use crate::ipc::SharedMemory;
use crate::protos::response::response::ResponseCode;
use crate::protos::{response, response::Response};
use anyhow::Result;
use protobuf::{EnumOrUnknown, MessageField};
use std::{collections::HashMap, net::SocketAddr};
use sysinfo::System;
use tokio::sync::mpsc;

type Tx = mpsc::Sender<Vec<u8>>;
type Rx = mpsc::Receiver<Vec<u8>>;

pub struct Info {
    pub used: u32,
    pub max: u32,
}

impl Into<Response> for Info {
    fn into(self) -> Response {
        let mut response_info = response::Info::new();
        response_info.used = self.used;
        response_info.max = self.max;

        let mut response = Response::new();
        response.set_info(response_info);
        response.code = EnumOrUnknown::from(ResponseCode::OK);

        return response;
    }
}

struct SystemStats {
    total_memory: u64,
    free_memory: u64,
    load: f64,
}

impl Into<response::stats::System> for SystemStats {
    fn into(self) -> response::stats::System {
        let mut system = response::stats::System::new();

        system.total_memory = self.total_memory;
        system.load = self.load;
        system.free_memory = self.free_memory;

        return system;
    }
}

struct SystemHappening {
    pid: u32,
    map_name: String,
    port: u16,
    password: String,
}

impl Into<response::stats::Happening> for SystemHappening {
    fn into(self) -> response::stats::Happening {
        let mut happening = response::stats::Happening::new();

        happening.port = self.port as u32;
        happening.pid = self.pid;
        happening.map_name = self.map_name;
        happening.password = self.password;

        return happening;
    }
}

pub struct Stats {
    system: SystemStats,
    happenings: Vec<SystemHappening>,
}

impl Into<Response> for Stats {
    fn into(self) -> Response {
        let mut stats = response::Stats::new();

        stats.system = MessageField::from(Some(self.system.into()));
        stats.happenings = self.happenings.into_iter().map(|h| h.into()).collect();

        let mut response = response::Response::new();
        response.set_stats(stats);
        response.code = EnumOrUnknown::from(ResponseCode::OK);

        return response;
    }
}

#[derive(Debug, Clone)]
struct Server {
    port: u16,
    happening_id: usize,
    map_name: String,
    pid: u32,
    password: String,
}

#[derive(Debug)]
pub struct State {
    pub shared_memory: SharedMemory,
    peers: HashMap<SocketAddr, Tx>,
    servers: Vec<Option<Server>>,
    sys: System,
    max: u32,
    in_use: u32,
}

impl State {
    pub fn new(max: u32) -> Self {
        let mut servers = Vec::with_capacity(max as usize);
        let sys = System::new_all();
        servers.resize(max as usize, None);

        return Self {
            servers,
            shared_memory: SharedMemory::new(max as usize),
            peers: HashMap::new(),
            sys,
            max,
            in_use: 0,
        };
    }

    pub async fn broadcast_msg(&mut self, msg: Vec<u8>) {
        for (_, sender) in self.peers.iter_mut() {
            sender.send(msg.clone()).await.unwrap();
        }
    }

    pub fn add_peer(&mut self, socket_addr: SocketAddr, tx: Tx) {
        self.peers.insert(socket_addr, tx);
    }

    pub fn remove_peer(&mut self, socket_addr: &SocketAddr) {
        self.peers.remove(socket_addr);
    }

    pub fn empty_index(&mut self) -> Option<usize> {
        return self
            .servers
            .iter()
            .enumerate()
            .find(|(_, srv)| srv.is_none())
            .map(|(i, _)| i);
    }

    pub fn add_server(
        &mut self,
        id: usize,
        happening_id: usize,
        port: u16,
        pid: u32,
        password: String,
        map_name: String,
    ) {
        self.servers[id] = Some(Server {
            happening_id,
            port,
            pid,
            password,
            map_name,
        });

        self.in_use += 1;
    }

    pub fn remove_server(&mut self, happening_id: usize) -> Option<usize> {
        for i in 0..self.servers.len() {
            if let Some(server) = &self.servers[i] {
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

    pub fn stats(&mut self) -> Stats {
        self.sys.refresh_all();

        let system = SystemStats {
            free_memory: self.sys.free_memory(),
            total_memory: self.sys.total_memory(),
            load: System::load_average().one,
        };
        let happenings: Vec<SystemHappening> = self
            .servers
            .clone()
            .into_iter()
            .filter(|srv| srv.is_some())
            .map(|srv| srv.unwrap())
            .map(|srv| SystemHappening {
                pid: srv.pid,
                port: srv.port,
                map_name: srv.map_name.clone(),
                password: srv.password.clone(),
            })
            .collect();

        return Stats { system, happenings };
    }

    pub fn is_port_taken(&self, port: u16) -> bool {
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
