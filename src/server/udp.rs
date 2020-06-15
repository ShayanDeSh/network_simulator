use std::net::UdpSocket;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::Mutex;
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
    pub hosts: &'a mut Vec<Host>,
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
        hosts: &'b mut Vec<Host>,
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

    pub fn listen(&self) -> (thread::JoinHandle<u32>, thread::JoinHandle<u32>) {
        let soc = self.socket.try_clone().expect("Could not clone");
        let (tx, rx): (mpsc::Sender<(usize, [u8; BUFFER_SIZE])>,
        mpsc::Receiver<(usize, [u8; BUFFER_SIZE])>) = mpsc::channel();
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
                        println!("got disc");
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


    pub fn send_discovery(&self, header: Header) {
        let mut counter = 0;
        let mut flag = true;
        while flag {
            let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE]; 
            let mut current: u16 = Server::copy_header(&mut buf, &header);
            let mut remained_buffer: i32 = USEFUL_BUFFER_SIZE as i32;
            for i in counter..self.hosts.len() {
                remained_buffer -= mem::size_of::<Host>() as i32;
                if remained_buffer < 0 {
                    counter = i;
                    break;
                }
                if i == self.hosts.len() - 1 {
                    flag = false;
                }
                let name_len = self.hosts[i].name.len() as u8;
                buf[current as usize] = name_len;
                current += 1;
                copy_str(&mut buf, current, &self.hosts[i].name);
                current += name_len as u16;
                copy_ip(&mut buf, current, &self.hosts[i].ipaddr);
                current += 4;
                copy_u16(&mut buf, current, self.hosts[i].port);
            }
            self.send(&header.dest_ip, header.dest_port, buf);
        }
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
        let request = extract_request(&data, 0, 4).trim().to_string(); 
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

    pub fn send(&self , ipaddr: &str,port: u16, buf: [u8; BUFFER_SIZE]) {
        let socket  = self.socket.try_clone()
            .expect("Could not clone socket for sending");
        let ip = format!("{}:{}", ipaddr, port);
        socket.send_to(&buf, ip)
            .expect("Could not send");
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

fn extract_request(data: &[u8], start: usize, end: usize) -> &str {
    std::str::from_utf8(&data[start..end]).expect("Could not extract str")
}

fn extract_u16(data: &[u8], start: usize) -> u16 {
    ((data[start] as u16) << 8) + data[start + 1] as u16 
}

fn extract_ip(data: &[u8], start: usize) -> String {
    format!("{}.{}.{}.{}", data[start], data[start + 1], data[start + 2], data[start + 3])
}
