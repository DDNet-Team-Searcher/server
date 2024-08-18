use crate::protos::response::response::{Data, ResponseCode};
use crate::protos::{response, response::Response};
use std::path::Path;
use std::{collections::HashMap, net::SocketAddr};
use sysinfo::System;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

type Tx = mpsc::Sender<Vec<u8>>;

pub struct Info {
    pub used: u32,
    pub max: u32,
}

impl Into<Response> for Info {
    fn into(self) -> Response {
        let info = response::Info {
            used: self.used,
            max: self.max,
            ..Default::default()
        };

        Response {
            code: ResponseCode::OK.into(),
            data: Some(Data::Info(info)),
            ..Default::default()
        }
    }
}

struct SystemStats {
    total_memory: u64,
    free_memory: u64,
    load: f64,
}

impl Into<response::stats::System> for SystemStats {
    fn into(self) -> response::stats::System {
        response::stats::System {
            total_memory: self.total_memory,
            load: self.load,
            free_memory: self.free_memory,
            ..Default::default()
        }
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
        response::stats::Happening {
            port: self.port as u32,
            pid: self.pid,
            map_name: self.map_name,
            password: self.password,
            ..Default::default()
        }
    }
}

pub struct Stats {
    system: SystemStats,
    happenings: Vec<SystemHappening>,
}

impl Into<Response> for Stats {
    fn into(self) -> Response {
        let stats = response::Stats {
            system: Some(self.system.into()).into(),
            happenings: self.happenings.into_iter().map(|h| h.into()).collect(),
            ..Default::default()
        };

        response::Response {
            data: Some(Data::Stats(stats)),
            code: ResponseCode::OK.into(),
            ..Default::default()
        }
    }
}

#[derive(Debug)]
pub struct Server {
    pub port: u16,
    pub happening_id: usize,
    pub map_name: String,
    pub pid: u32,
    pub password: String,
    fifo: tokio::fs::File,
}

impl Server {
    pub async fn shutdown(&mut self) {
        //self.fifo.write_all(b"shutdown \"reason\"").await.unwrap();
        self.fifo.write_all(b"shutdown Foo").await.unwrap();
    }
}

#[derive(Debug)]
pub struct State {
    peers: HashMap<SocketAddr, Tx>,
    servers: Vec<Option<Server>>,
    sys: System,
    max: u32,
    in_use: u32,
}

impl State {
    pub fn new(max: u32) -> Self {
        Self {
            servers: Vec::from_iter((0..max).map(|_| None)),
            peers: HashMap::new(),
            sys: System::new_all(),
            max,
            in_use: 0,
        }
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
        self.servers
            .iter()
            .enumerate()
            .find(|(_, srv)| srv.is_none())
            .map(|(i, _)| i)
    }

    pub async fn add_server(
        &mut self,
        id: usize,
        happening_id: usize,
        port: u16,
        pid: u32,
        password: String,
        map_name: String,
        fifo_path: &Path,
    ) {
        let fifo = tokio::fs::File::create(fifo_path).await.unwrap();

        self.servers[id] = Some(Server {
            happening_id,
            port,
            pid,
            password,
            map_name,
            fifo,
        });
        self.in_use += 1;
    }

    pub fn remove_server(&mut self, happening_id: usize) -> Option<Server> {
        for i in 0..self.servers.len() {
            if let Some(server) = &self.servers[i] {
                if server.happening_id == happening_id {
                    let server = std::mem::take(&mut self.servers[i]).unwrap();

                    self.servers[i] = None;
                    self.in_use -= 1;

                    return Some(server);
                }
            }
        }

        None
    }

    pub fn get_info(&self) -> Info {
        Info {
            used: self.in_use,
            max: self.max,
        }
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
            .iter()
            .filter(|srv| srv.is_some())
            .map(|srv| srv.as_ref().unwrap())
            .map(|srv| SystemHappening {
                pid: srv.pid,
                port: srv.port,
                map_name: srv.map_name.clone(),
                password: srv.password.clone(),
            })
            .collect();

        Stats { system, happenings }
    }

    pub fn is_port_taken(&self, port: u16) -> bool {
        for server in &self.servers {
            if let Some(srv) = server {
                if srv.port == port {
                    return true;
                }
            }
        }

        false
    }
}
