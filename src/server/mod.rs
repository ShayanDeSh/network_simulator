use std::collections::HashMap;

use std::fs;
use std::thread;
use std::sync::{Arc, RwLock};
use std::io;
use std::net::{IpAddr, UdpSocket};

mod udp;
mod tcp;
mod header;
mod host;

pub fn start(port: String, location: String, dir: String, ip: String) {
    let hosts: Arc<RwLock<HashMap<String, RwLock<host::Host>>>> = 
        Arc::new(RwLock::new(HashMap::new()));
    let requests: Arc<RwLock<Vec<(String, String)>>> = 
        Arc::new(RwLock::new(Vec::new()));
    if !location.is_empty() {
        read_hosts(hosts.clone(), &location);
    }
    let connection = udp::Server::init(&port,
        hosts.clone(), &ip, requests.clone());
    let socket = connection.socket.try_clone()
    .expect("Could not clone socket");
    let _requests = requests.clone();
    let port: u16 = port.parse().unwrap();
    input(hosts.clone(), socket, port, ip.clone(), _requests);
    let (process_handler, listen_handler, discover_handler) 
        = connection.listen(dir);
    process_handler.join().unwrap();
    listen_handler.join().unwrap();
    discover_handler.join().unwrap();
}


fn input(
    hosts: Arc<RwLock<HashMap<String, RwLock<host::Host>>>>,
    socket: UdpSocket,
    port: u16,
    ip: String,
    requests: Arc<RwLock<Vec<(String, String)>>>,
    ) {
    thread::spawn(move || {
        loop {
            let mut input = String::new();
            io::stdin().read_line(&mut input)
                .expect("Something Went wrong on reading from input");
            let input = input.trim();
            match input  {
                "list" => {
                    let hosts = hosts.read().unwrap();
                    list(&hosts);
                },
                "get" => {
                    let mut input = String::new();
                    io::stdin().read_line(&mut input)
                        .expect("Something Went wrong on reading from input");
                    input = input.trim().to_string();
                    udp::Server::get(&socket, &input, hosts.clone(), 
                        port,
                        &ip, requests.clone()); 
                }, 
                _ => {
                    continue;
                }
            }
        }
    });
}


// List command
fn list(hosts: &HashMap<String, RwLock<host::Host>>) {
    for (_, host) in hosts {
        let host = host.read().unwrap();
        println!("------------------------------------------------------------");
        println!("Name: {:?}", host.name);
        println!("IP adrr.: {:?}", host.ipaddr);
        println!("Port: {}", host.port);
    }
}

// Reading list of host from the file provided using -l
fn read_hosts(
    hosts: Arc<RwLock<HashMap<String, RwLock<host::Host>>>>,
    location: &str
    ) {
    let mut hosts = hosts.write().unwrap();
    let raw_hosts = fs::read_to_string(location)
        .expect("could not read hosts form file");
    let mut raw_hosts: Vec<&str> = raw_hosts.split("\n").collect();
    raw_hosts.pop();
    for raw_host in raw_hosts {
        let host:Vec<&str> = raw_host.split(" ").collect();
        let ip: IpAddr = host[1].parse().unwrap();
        let gateway = !ip.is_loopback();
        let key = format!("{}:{}", host[1], host[2]);
        let host = host::Host::new(
        host[0].to_string(),
        host[1].to_string(),
        host[2].parse::<u16>().expect("not parsable"),
        gateway
        );
        hosts.insert(key, host);
    }
}

