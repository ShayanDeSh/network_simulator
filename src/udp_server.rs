use std::net::UdpSocket;

struct UdpCon {
    socket: UdpSocket
}


impl UdpCon {
    fn init() -> UdpCon {
        let socket  = UdpSocket::bind("127.0.0.1:8080")
            .expect("Something went wrong while trying to create UDP socket!!");
        UdpCon {
            socket
        }
    }

    fn listen(&self) -> (usize, std::net::SocketAddr, [u8; 2048]) {
        let mut buf = [0; 2048];
        let (amt, src) = self.socket.recv_from(&mut buf)
            .expect("shit happened");
        (amt, src, buf)
    }

    fn send_discovery() {
    }

}


fn main() {
    println!("Hello, world!");
    let connection = UdpCon::init();
    loop {
        let (amt, src, buf) = connection.listen();
        let s = std::str::from_utf8(&buf[..amt])
            .expect("Something happened while converting from utf8 to string!!");
        println!("recived {:?}", s);
    }
}
