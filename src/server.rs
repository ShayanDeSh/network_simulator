use std::collections::HashMap;
use std::fs;
use std::thread;
use std::sync::{Mutex, Arc, RwLock};
use std::io;

mod udp;

pub fn start(port: String, location: String) {
    let table: HashMap<String, String> = HashMap::new();
    let rtable = Mutex::new(table);
    let hosts: Arc<RwLock<HashMap<String, udp::Host>>> = 
        Arc::new(RwLock::new(HashMap::new()));
    if !location.is_empty() {
        read_hosts(hosts.clone(), &location);
    }
    let list_clone =  hosts.clone();
    let connection = udp::Server::init(&port, rtable,
        hosts.clone(), "127.0.0.1");
    let socket = connection.socket.try_clone()
    .expect("Could not clone socket");
    thread::spawn(move || {
        loop {
            let mut input = String::new();
            io::stdin().read_line(&mut input)
                .expect("Something Went wrong on reading from input");
            input = input.trim().to_string();
            let list = "list".to_string();
            let get = "get".to_string();
            match input  {
                list => {
                    let hosts = list_clone.read().unwrap();
                    print_hosts(&hosts);
                },
                get => {
                    let mut input = String::new();
                    io::stdin().read_line(&mut input)
                        .expect("Something Went wrong on reading from input");
                    input = input.trim().to_string();
                    udp::Server::get(&socket, &input, hosts.clone(), 
                        port.parse::<u16>().unwrap(), "127.0.0.1"); 
                }
            }
        }
    });
    let (process_handler, listen_handler) = connection.listen();
    process_handler.join().unwrap();
    listen_handler.join().unwrap();
}

fn print_hosts(hosts: &HashMap<String, udp::Host>) {
    for (_, host) in hosts {
        println!("Name: {:?}", host.name);
        println!("IP adrr.: {:?}", host.ipaddr);
        println!("Port: {}", host.port);
    }
}

fn read_hosts(hosts: Arc<RwLock<HashMap<String, udp::Host>>>, location: &str) {
    let mut hosts = hosts.write().unwrap();
    let raw_hosts = fs::read_to_string(location)
        .expect("could not read hosts form file");
    let mut raw_hosts: Vec<&str> = raw_hosts.split("\n").collect();
    raw_hosts.pop();
    for raw_host in raw_hosts {
        let host:Vec<&str> = raw_host.split(" ").collect();
        let key = format!("{}:{}", host[1], host[2]);
        let host = udp::Host::new(
        host[0].to_string(),
        host[1].to_string(),
        host[2].parse::<u16>().expect("not parsable")
        );
        hosts.insert(key, host);
    }
}

