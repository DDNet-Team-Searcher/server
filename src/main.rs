use std::process;
use std::process::{ Command };
use std::net::{ TcpListener, TcpStream };
use std::io::{ prelude::*, BufReader };
use serde::{Deserialize, Serialize};
use serde_json;
use std::sync::{ Mutex, Arc };
use std::thread;

#[derive(Serialize, Deserialize, Debug)]
struct StartResponseSuccess<'a> {
    status: &'a str,
    pid: u32,
    id: u32,
    password: String,
    port: u32
}

#[derive(Serialize, Deserialize, Debug)]
struct ShutdownResponseSuccess<'a> {
    status: &'a str,
    pid: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct StartRequest {
    map_name: String,
    password: String,
    config_file: String,
    id: u32,
    port: u32
}

impl StartRequest {
    fn start(&self) -> String {
        let password_str = format!("sv_port {}; password {}; sv_map {}", self.port, self.password, self.map_name);

        let child = Command::new("./DDnet-Server")
        .current_dir("/mnt/d/ddnet-team-searcher/ddnet-server")
        .args([&password_str, "-f", &self.config_file])
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null())
        .spawn()
        .unwrap();  

        let res = StartResponseSuccess {
            id: self.id,
            pid: child.id(),
            status: "SERVER_STARTED_SUCCESSFULLY",
            password: self.password.clone(),
            port: self.port
        };

        serde_json::to_string(&res).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct ShutdownRequest {
    pid: u32
}

impl ShutdownRequest {
    fn shutdown(&self) -> String {
        Command::new("kill")
        .args(["-9", &self.pid.to_string()])
        .output()
        .expect("Failed to execute process");


        let res = ShutdownResponseSuccess {
            status: "SERVER_SHUTDOWN_SUCCESSFULLY",
            pid: self.pid
        };

        serde_json::to_string(&res).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum Request {
    StartRequest(StartRequest),
    ShutdownRequest(ShutdownRequest),
}

fn main() {
    let allowed_ips = ["192.168.56.1"];

    let listener = TcpListener::bind("192.168.56.1:9090").unwrap();

    let connected_ppl_ptr = Arc::new(Mutex::new(Vec::<TcpStream>::new()));

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        
        let slice = &stream.peer_addr().unwrap().ip().to_string()[..];
        
        if !allowed_ips.contains(&slice) {
            stream.write_all("{\"status\":\"ACCESS_DENIED\"}".as_bytes()).unwrap();
            continue;
        }

        let stream_copy = stream.try_clone().unwrap();
        let connected_ppl_ptr_copy = connected_ppl_ptr.clone();

        thread::spawn(|| {
            handle_conn(stream, connected_ppl_ptr_copy);
        });
    
        let mut guard = connected_ppl_ptr.lock().unwrap();
        let data = &mut *guard;

        data.push(stream_copy);
    }
}

fn handle_conn(mut stream: TcpStream, connected_ppl: Arc<Mutex<Vec<TcpStream>>>) {
    loop {
        let mut line = String::new();
        let mut buf_reader = BufReader::new(&mut stream);
        
        buf_reader.read_line(&mut line).unwrap();

        let json: serde_json::Result<Request> = serde_json::from_str(&line);

        let mut guard = connected_ppl.lock().unwrap();
        let connected_users = &mut *guard;

        if line.as_bytes().len() == 0 {
            let index = connected_users.iter().position(|r| r.peer_addr().unwrap() == stream.peer_addr().unwrap()).unwrap();
            connected_users.remove(index);

            // user disconected
            break;
        }

        match json {
            Ok(data) => {
                match data {
                    Request::StartRequest(start_server) => {
                        let res = start_server.start();

                        for user in connected_users {
                            user.write_all(res.as_bytes()).unwrap();
                        }
                    },
                    Request::ShutdownRequest(shutdown_server) => {
                        let res = shutdown_server.shutdown();
                    
                        for user in connected_users {
                            user.write_all(res.as_bytes()).unwrap();
                        }
                    },
                }
            },
            Err(_) => {
                stream.write_all("{status: \"BAD_DATA\"}".as_bytes()).unwrap();
            }
        }
    }
}