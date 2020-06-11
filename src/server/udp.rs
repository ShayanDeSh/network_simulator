use std::net::UdpSocket;
use std::collections::HashMap;

pub struct Con {
    socket: UdpSocket,
    rtable: HashMap<String, String>
}


impl Con {
    pub fn init(port: String) -> Con {
        let socket  = UdpSocket::bind(format!("127.0.0.1:{}", port))
            .expect("Something went wrong while trying to create UDP socket!!");
        let rtable  = HashMap::new();
        Con {
            socket,
            rtable
        }
    }

    pub fn listen(&self) -> (usize, std::net::SocketAddr, [u8; 2048]) {
        let mut buf = [0; 2048];
        let (amt, src) = self.socket.recv_from(&mut buf)
            .expect("shit happened");
        (amt, src, buf)
    }

    pub fn send_discovery(&self) { 
        let data    = String::from("Bojack Horseman");
        let data_buffer = data.as_bytes();
        self.socket.send_to(data_buffer, "TODO")
            .expect("Something happened while sending discovery over UDP!!");
    }
}

