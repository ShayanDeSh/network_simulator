use std::net::UdpSocket;
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;

pub struct Server {
    socket: UdpSocket,
    rtable: HashMap<String, String>,
    tx: mpsc::Sender<(usize, [u8; 2048])>,
    rx: mpsc::Receiver<(usize, [u8; 2048])>
}


impl Server {
    pub fn init(port: String) -> Server {
        let socket  = UdpSocket::bind(format!("127.0.0.1:{}", port))
            .expect("Something went wrong while trying to create UDP socket!!");
        let rtable  = HashMap::new();
        let (tx, rx) = mpsc::channel();
        Server {
            socket,
            rtable,
            tx,
            rx 
        }
    }

    pub fn listen(self) -> (thread::JoinHandle<u32>, thread::JoinHandle<u32>) {
        let soc = self.socket;
        let tx  = self.tx;
        let rx  = self.rx;
        let process_handler = thread::spawn(move || {
            loop {
                let (amt, data) = rx.recv().unwrap();
                let st          = std::str::from_utf8(&data[..amt]);
                println!("{:?}", st);
            }
        });
        let listen_handler = thread::spawn(move || {
            loop {
                let mut buf = [0; 2048];
                let (amt, src) = soc.recv_from(&mut buf)
                    .expect("shit happened");
                tx.send((amt, buf)).unwrap();
            }
        });
        return (process_handler, listen_handler);
    }

}

