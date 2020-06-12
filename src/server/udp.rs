use std::net::UdpSocket;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::Mutex;
use std::thread;

pub struct Server {
    socket: UdpSocket,
    rtable: Mutex<HashMap<String, String>>,
}


impl Server {
    pub fn init(port: &str, rtable: Mutex<HashMap<String, String>>) -> Server {
        let socket  = UdpSocket::bind(format!("127.0.0.1:{}", port))
            .expect("Something went wrong while trying to create UDP socket!!");
        Server {
            socket,
            rtable,
        }
    }

    pub fn listen(&self) -> (thread::JoinHandle<u32>, thread::JoinHandle<u32>) {
        let soc = self.socket.try_clone().expect("Could not clone");
        let (tx, rx): (mpsc::Sender<(usize, [u8; 2048])>,
        mpsc::Receiver<(usize, [u8; 2048])>) = mpsc::channel();
        let process_handler = thread::spawn(move || {
            loop {
                let (amt, data) = rx.recv().unwrap();
                let request = std::str::from_utf8(&data[..8]);
                println!("{:?}", request);
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

pub fn send(port: &str) {
    let socket  = UdpSocket::bind(format!("127.0.0.1:{}", port))
        .expect("");
    socket.send_to("Bojac Horseman".as_bytes(),"127.0.0.1:8000")
        .expect("Could not send");
}

