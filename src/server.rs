use std::collections::HashMap;
use std::sync::Mutex;
use std::fs;
use std::thread;

mod udp;

pub fn start(port: String, location: String) {
    let mut hosts: Vec<udp::Host> = Vec::new();
    if !location.is_empty() {
        read_hosts(&mut hosts, &location);
    }
    let table: HashMap<String, String> = HashMap::new();
    let rtable = Mutex::new(table);
    let connection = udp::Server::init(&port, rtable, &mut hosts, "127.0.0.1");
    let (process_handler, listen_handler) = connection.listen();
    loop {
        thread::sleep_ms(10000);
        start_discovery(&connection);
    }
    process_handler.join().unwrap();
    listen_handler.join().unwrap();
}

fn read_hosts(hosts: &mut Vec<udp::Host>, location: &str) {
    let raw_hosts = fs::read_to_string(location)
        .expect("could not read hosts form file");
    let mut raw_hosts: Vec<&str> = raw_hosts.split("\n").collect();
    raw_hosts.pop();
    for raw_host in raw_hosts {
        let host:Vec<&str> = raw_host.split(" ").collect();
        let host = udp::Host::new(
        host[0].to_string(),
        host[1].to_string(),
        host[2].parse::<u16>().expect("not parsable")
        );
        hosts.push(host);
    }
}

fn start_discovery(connection: &udp::Server) {
    for (_, host) in connection.hosts.iter().enumerate() {
        let header = udp::Header::new("disc", host.port, connection.udp_port, &host.ipaddr, "127.0.0.1");
        connection.send_discovery(header);
    }
}
