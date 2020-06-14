use std::collections::HashMap;
use std::sync::Mutex;

mod udp;

pub fn start(port: String) {
    println!("Hello, world!");
    let table: HashMap<String, String> = HashMap::new();
    let rtable = Mutex::new(table);
    let connection = udp::Server::init(&port, rtable);
    let h = udp::Header {
        request: "disc".to_string(),
        dest_port: 8000,
        src_port: 8080,
        dest_ip: "127.0.0.1".to_string(),
        src_ip: "127.0.0.1".to_string()
    };
    connection.send_discovery(h);
    let (process_handler, listen_handler) = connection.listen();
    process_handler.join().unwrap();
    listen_handler.join().unwrap();
}
