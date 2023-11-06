use anyhow::Result;
use std::collections::HashMap;

pub struct State {
    servers: HashMap<usize, usize>,
}

impl State {
    pub fn new() -> Self {
        return Self {
            servers: HashMap::new(),
        };
    }

    pub fn add_server(&mut self, id: usize, pid: usize) -> Result<()> {
        if self.servers.contains_key(&id) {
            //TODO: return error
        }

        self.servers.insert(id, pid);
        return Ok(());
    }

    pub fn remove_server(&mut self, id: usize) -> Option<usize> {
        return self.servers.remove(&id);
    }

    pub fn is_port_taken(&self, port: usize) -> bool {
        return self.servers.values().any(|val| val == &port);
    }
}
