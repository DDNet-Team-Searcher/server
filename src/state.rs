use anyhow::Result;
use std::collections::HashMap;

pub struct Info {
    pub used: usize,
    pub max: usize,
}

pub struct State {
    servers: HashMap<usize, usize>,
    max: usize,
}

impl State {
    pub fn new(max: usize) -> Self {
        return Self {
            servers: HashMap::new(),
            max,
        };
    }

    pub fn add_server(&mut self, id: usize, pid: usize) -> Result<()> {
        if self.servers.len() == self.max {
            //TODO: return error
        }

        if self.servers.contains_key(&id) {
            //TODO: return error
        }

        self.servers.insert(id, pid);
        return Ok(());
    }

    pub fn get_info(&self) -> Info {
        return Info {
            used: self.servers.len(),
            max: self.max,
        };
    }

    pub fn remove_server(&mut self, id: usize) -> Option<usize> {
        return self.servers.remove(&id);
    }

    pub fn is_port_taken(&self, port: usize) -> bool {
        return self.servers.values().any(|val| val == &port);
    }
}
