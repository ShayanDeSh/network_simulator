use std::net::UdpSocket;

pub struct UdpCon {
    socket: UdpSocket
}


impl UdpCon {
    pub fn init() -> UdpCon {
        let socket  = UdpSocket::bind("127.0.0.1:8080")
            .expect("Something went wrong while trying to create UDP socket!!");
        UdpCon {
            socket
        }
    }

    pub fn listen(&self) -> (usize, std::net::SocketAddr, [u8; 2048]) {
        let mut buf = [0; 2048];
        let (amt, src) = self.socket.recv_from(&mut buf)
            .expect("shit happened");
        (amt, src, buf)
    }

    pub fn send_discovery() {
    }

}

