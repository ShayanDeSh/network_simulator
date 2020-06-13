use std::net::UdpSocket;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::Mutex;
use std::thread;

const BUFFER_SIZE: usize = 2048;

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
        let (tx, rx): (mpsc::Sender<(usize, [u8; BUFFER_SIZE])>,
        mpsc::Receiver<(usize, [u8; BUFFER_SIZE])>) = mpsc::channel();
        let process_handler = thread::spawn(move || {
            loop {
                let (amt, data) = rx.recv().unwrap();
                let request = std::str::from_utf8(&data[..4])
                    .expect("Not parsable request type");
                let request =  request.trim();
                let dest_port: u16 = ((data[4] as u16) << 8) + data[5] as u16; 
                let src_port: u16 = ((data[6] as u16) << 8) + data[7] as u16; 
                let dest_ip: String = format!("{}.{}.{}.{}",
                    data[8], data[9], data[10], data[11]);
                let src_ip: String = format!("{}.{}.{}.{}",
                    data[12], data[13], data[14], data[15]);
                match request {
                    "get" => {
                        println!("got get :)");
                    },
                    "list" => {
                        println!("got list");
                    }
                    _ => {
                        continue;
                    }
                }
                println!("{:?}", request);
                println!("{:?}", dest_port);
                println!("{:?}", src_port);
                println!("{:?}", dest_ip);
                println!("{:?}", src_ip);
            }
        });
        let listen_handler = thread::spawn(move || {
            loop {
                let mut buf = [32; BUFFER_SIZE];
                let (amt, src) = soc.recv_from(&mut buf)
                    .expect("shit happened");
                tx.send((amt, buf)).unwrap();
            }
        });
        return (process_handler, listen_handler);
    }

    pub fn send(&self , ipaddr: &str, buf: [u8; BUFFER_SIZE]) {
        let socket  = self.socket.try_clone()
            .expect("Could not clone socket for sending");
        socket.send_to(&buf, ipaddr)
            .expect("Could not send");
    }
}


