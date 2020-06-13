use std::net::UdpSocket;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::Mutex;
use std::thread;
use std::mem;

const BUFFER_SIZE: usize = 2048;
const USEFUL_BUFFER_SIZE: usize = BUFFER_SIZE - 16;

pub struct Host {
    name: String,
    ipaddr: String,
    port: u16
}

pub struct Server {
    socket: UdpSocket,
    hosts: Vec<Host>,
    rtable: Mutex<HashMap<String, String>>,
}

pub struct Header {
    request: String,
    dest_port: u16,
    src_port: u16,
    dest_ip: String,
    src_ip: String
}


impl Server {
    pub fn init(port: &str, rtable: Mutex<HashMap<String, String>>) -> Server {
        let socket  = UdpSocket::bind(format!("127.0.0.1:{}", port))
            .expect("Something went wrong while trying to create UDP socket!!");
        let hosts: Vec<Host> = Vec::new();
        Server {
            socket,
            hosts,
            rtable
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


    fn send_discovery(&self, header: Header) {
        let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE]; 
        copy_str(&mut buf, 0, &header.request);
        let req_byte = header.request.as_bytes();
        for (i, byte) in req_byte.iter().enumerate() {
            buf[i] = *byte;
        }
        let dport_bytes = header.dest_port.to_be_bytes();
        buf[4] = dport_bytes[0];
        buf[5] = dport_bytes[1];
        let sport_bytes = header.src_port.to_be_bytes();
        buf[6] = sport_bytes[0];
        buf[7] = sport_bytes[1];
        let dip = header.dest_ip.replace(".", "");
        let dip = dip.as_bytes();
        for (i, num) in dip.iter().enumerate() {
            buf[8 + i] = *num;
        }
        let sip = header.src_ip.replace(".", "");
        let sip = sip.as_bytes();
        for (i, num) in sip.iter().enumerate() {
            buf[12 + i] = *num;
        }
        let mut remained_buffer: i32 = USEFUL_BUFFER_SIZE as i32;
        let mut current = 16;
        for host in &self.hosts {
            remained_buffer -= mem::size_of::<Host>() as i32;
            if remained_buffer < 0 {
                break;
            }
            let name_len = host.name.len() as u8;
            buf[current] = name_len;
            current += 1;
        }
    }

    pub fn send(&self , ipaddr: &str, buf: [u8; BUFFER_SIZE]) {
        let socket  = self.socket.try_clone()
            .expect("Could not clone socket for sending");
        socket.send_to(&buf, ipaddr)
            .expect("Could not send");
    }

}

fn copy_str(buf: &mut [u8], current: u16, string: &str) {
    let req_byte = string.as_bytes();
    for (i, byte) in req_byte.iter().enumerate() {
        buf[current as usize + i] = *byte;
    }
}
