use std::collections::HashMap;
use std::sync::Mutex;

mod udp;

pub fn start(port: String) {
    println!("Hello, world!");
    let table: HashMap<String, String> = HashMap::new();
    let rtable = Mutex::new(table);
    let connection = udp::Server::init(&port, rtable);
    let (process_handler, listen_handler) = connection.listen();
    process_handler.join().unwrap();
    listen_handler.join().unwrap();
}
