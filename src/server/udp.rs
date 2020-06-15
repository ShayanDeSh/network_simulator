use std::net::UdpSocket;

use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::{Mutex, Arc, RwLock};
use std::thread;
use std::mem;

const BUFFER_SIZE: usize = 2048;
const USEFUL_BUFFER_SIZE: usize = BUFFER_SIZE - 16;

pub struct Host {
    pub name: String,
    pub ipaddr: String,
    pub port: u16
}

pub struct Server<'a> {
    socket: UdpSocket,
    pub hosts: &'a mut HashMap<String, Host>,
    rtable: Mutex<HashMap<String, String>>,
    pub udp_port: u16,
    pub ipaddr: String
}

pub struct Header {
    pub request: String,
    pub dest_port: u16,
    pub src_port: u16,
    pub dest_ip: String,
    pub src_ip: String
}


impl Header {
    pub fn new(request: &str, dest_port: u16,
            src_port: u16, dest_ip: &str, src_ip: &str) -> Header {
        let request = request.to_string();
        let dest_ip = dest_ip.to_string();
        let src_ip = src_ip.to_string();
        Header {
            request,
            dest_port,
            src_port,
            dest_ip,
            src_ip
        }
    }
}


impl Host {
    pub fn new(name: String, ipaddr: String, port: u16) -> Host {
        Host {
            name,
            ipaddr,
            port
        }
    }
}

impl<'a> Server<'a> {
    pub fn init<'b>(
        udp_port: &str,
        rtable: Mutex<HashMap<String, String>>,
        hosts: &'b mut HashMap<String, Host>,
        ipaddr: &str
        ) -> Server<'b> {
        let socket  = UdpSocket::bind(format!("127.0.0.1:{}", udp_port))
            .expect("Something went wrong while trying to create UDP socket!!");
        let udp_port = udp_port.parse::<u16>().expect("non parsable port");
        let ipaddr = ipaddr.to_string();
        Server {
            socket,
            hosts,
            rtable,
            udp_port,
            ipaddr
        }
    }

    pub fn listen<'b>(self) -> (thread::JoinHandle<u32>, thread::JoinHandle<u32>) {
        let soc = self.socket.try_clone().expect("Could not clone");
        let soc2 = self.socket.try_clone().expect("Could not clone");
        let (tx, rx): (mpsc::Sender<(usize, [u8; BUFFER_SIZE])>,
        mpsc::Receiver<(usize, [u8; BUFFER_SIZE])>) = mpsc::channel();
//        let x:&'a mut HashMap<String, Host> = self.hosts;
        let x: Arc<RwLock<&'a mut HashMap<String, Host>>> = Arc::new(RwLock::new(self.hosts));
        let y = x.clone();
        let udp_p: u16 = self.udp_port.clone();
        let discover_handler = thread::spawn(move || {
            loop {
                thread::sleep_ms(10000);
                let hosts = x.read().unwrap();
                for (key, host) in hosts.iter() {
                    let header = Header::new("disc", host.port, udp_p, &host.ipaddr, "127.0.0.1");
                    Server::send_discovery(&soc2, x, header);
                }
            }
        });
        let process_handler = thread::spawn(move || {
            loop {
                let (amt, data) = rx.recv().unwrap();
                let header = Server::extract_header(&data);
                let request:&str = &header.request;
                match request {
                    "get" => {
                        println!("got get :)");
                    },
                    "list" => {
                        println!("got list");
                    },
                    "disc" => {
                        Server::extract_disc_body(y, &data, 16, amt - 16);
                    },
                    _ => {
                        continue;
                    }
                }
                println!("{:?}", request);
                println!("{:?}", header.dest_port);
                println!("{:?}", header.src_port);
                println!("{:?}", header.dest_ip);
                println!("{:?}", header.src_ip);
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

    fn extract_disc_body(hosts: Arc<RwLock<&'a mut HashMap<String, Host>>> , data: &[u8], current: usize, end: usize) {
        let mut current = current;
        let mut hosts = hosts.write().unwrap();
        while current < end { 
            let name_len = data[current];
            current += 1;
            let name = extract_str(data, current, current + name_len as usize);
            current += name_len as usize;
            let ipaddr = extract_ip(data, current);
            current += 4;
            let port = extract_u16(data, current);
            current += 2;
            let key = format!("{}:{}", ipaddr, port);
            if !hosts.contains_key(&key) {
                let host = Host::new(name.to_string(), ipaddr, port);
                hosts.insert(key, host); 
            }
        }
    }


    pub fn send_discovery(socket: &UdpSocket, hosts: Arc<RwLock<&'a mut HashMap<String, Host>>>, header: Header) {
        let mut counter = 0;
        let mut flag = true;
        let hosts = hosts.read().unwrap();
        let hosts: Vec<&Host> = hosts.iter().map(|(_, host)| host).collect();
        while flag {
            let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE]; 
            let mut current: u16 = Server::copy_header(&mut buf, &header);
            let mut remained_buffer: i32 = USEFUL_BUFFER_SIZE as i32;
            for i in counter..hosts.len() {
                remained_buffer -= mem::size_of::<Host>() as i32;
                if remained_buffer < 0 {
                    counter = i;
                    break;
                }
                if i == hosts.len() - 1 {
                    flag = false;
                }
                let name_len = hosts[i].name.len() as u8;
                buf[current as usize] = name_len;
                current += 1;
                copy_str(&mut buf, current, &hosts[i].name);
                current += name_len as u16;
                copy_ip(&mut buf, current, &hosts[i].ipaddr);
                current += 4;
                copy_u16(&mut buf, current, hosts[i].port);
            }
            Server::send(&socket, &header.dest_ip, header.dest_port, buf);
        }
    }

    pub fn send(socket: &UdpSocket, ipaddr: &str,port: u16, buf: [u8; BUFFER_SIZE]) {
        let ip = format!("{}:{}", ipaddr, port);
        socket.send_to(&buf, ip)
            .expect("Could not send");
    }

    fn copy_header(buf: &mut [u8], header: &Header) -> u16 {
            copy_str(buf, 0, &header.request);
            copy_u16(buf, 4, header.dest_port);
            copy_u16(buf, 6, header.src_port);
            copy_ip(buf, 8, &header.dest_ip);
            copy_ip(buf, 12, &header.src_ip);
            return 16;
    }

    fn extract_header(data: &[u8]) -> Header {
        let request = extract_str(&data, 0, 4).trim().to_string(); 
        let dest_port = extract_u16(&data, 4);
        let src_port = extract_u16(&data, 6);
        let dest_ip = extract_ip(&data, 8);
        let src_ip = extract_ip(&data, 12);
        Header {
            request,
            dest_port,
            src_port,
            dest_ip,
            src_ip
        }
    }

}

fn copy_str(buf: &mut [u8], current: u16, string: &str) {
    let string = string.as_bytes();
    for (i, byte) in string.iter().enumerate() {
        buf[current as usize + i] = *byte;
    }
}

fn copy_u16(buf: &mut [u8], current: u16, num: u16) {
    let num = num.to_be_bytes();
    buf[current as usize] = num[0];
    buf[current as usize + 1] = num[1];
}

fn copy_ip(buf: &mut [u8], current: u16, ip: &str) {
    let ip: Vec<&str>= ip.split(".").collect();
    for (i, num) in ip.iter().enumerate() {
        buf[current as usize + i] = num.parse::<u8>().expect("Wrong ip");
    }
}

fn extract_str(data: &[u8], start: usize, end: usize) -> &str {
    std::str::from_utf8(&data[start..end]).expect("Could not extract str")
}

fn extract_u16(data: &[u8], start: usize) -> u16 {
    ((data[start] as u16) << 8) + data[start + 1] as u16 
}

fn extract_ip(data: &[u8], start: usize) -> String {
    format!("{}.{}.{}.{}", data[start], data[start + 1], data[start + 2], data[start + 3])
}

