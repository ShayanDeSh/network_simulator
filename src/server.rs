mod udp;

pub fn start(port: String) {
    println!("Hello, world!");
    let connection = udp::Server::init(port);
    let (process_handler, listen_handler) = connection.listen();
    process_handler.join().unwrap();
    listen_handler.join().unwrap();
}
