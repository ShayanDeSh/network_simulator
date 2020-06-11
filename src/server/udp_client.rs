use std::net::UdpSocket;

let a = 4;

fn main() {
    let socket  = UdpSocket::bind("127.0.0.1:8000").expect("shit happened");
    let data    = String::from("Bojack Horseman");
    let data_buffer = data.as_bytes();
    socket.send_to(data_buffer, "127.0.0.1:8080").expect("shit happened");
}
