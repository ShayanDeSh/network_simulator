use std::net::UdpSocket;
use std::io::prelude::*;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::{Arc, RwLock};
use std::thread;
use std::mem;
use std::time::Duration;
use std::fs;
use std::net::{TcpListener, TcpStream, IpAddr};
use crate::bytes;
use crate::server::tcp;

const BUFFER_SIZE: usize = 8192;
const USEFUL_BUFFER_SIZE: usize = BUFFER_SIZE - 16;
pub const MAX_CONEECTION: u16 = 20;

pub struct Host {
    pub name: String,
    pub ipaddr: String,
    pub port: u16,
    pub num_requests: u16,
    pub gateway: bool
}

pub struct Server {
    pub socket: UdpSocket,
    pub hosts: Arc<RwLock<HashMap<String, RwLock<Host>>>>,
    pub requests: Arc<RwLock<Vec<(String, String)>>>,
    pub udp_port: u16,
    pub ipaddr: String,
    pub connection_num: Arc<RwLock<u16>>,
    pub gateway: bool
}

pub struct Header {
    pub request: String,
    pub dest_port: u16,
    pub src_port: u16,
    pub dest_ip: String,
    pub src_ip: String
}


impl Header {
    pub fn new(
        request: &str, dest_port: u16,
        src_port: u16,
        dest_ip: &str,
        src_ip: &str
        ) -> Header {
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
    pub fn new(
        name: String, 
        ipaddr: String, 
        port: u16, 
        gateway: bool
        ) -> RwLock<Host> {
        let num_requests = 0;
        let host = Host {
            name,
            ipaddr,
            port,
            num_requests,
            gateway
        };
        RwLock::new(host)
    }
}

impl Server {
    pub fn init(
        udp_port: &str,
        hosts: Arc<RwLock<HashMap<String, RwLock<Host>>>>,
        ipaddr: &str,
        requests: Arc<RwLock<Vec<(String, String)>>>,
        ) -> Server {
        let socket  = UdpSocket::bind(format!("{}:{}", ipaddr,udp_port))
            .expect("Something went
                wrong while trying to create UDP socket!!");
        let udp_port = udp_port.parse::<u16>().expect("non parsable port");
        let ipaddr = ipaddr.to_string();
        let connection_num = Arc::new(RwLock::new(0));
        let addr: IpAddr = ipaddr.parse().unwrap();
        let gateway = !addr.is_loopback();
        Server {
            socket,
            hosts,
            requests,
            udp_port,
            ipaddr,
            connection_num,
            gateway
        }
    }

    pub fn listen(self, dir: String) -> (
            thread::JoinHandle<u32>,
            thread::JoinHandle<u32>,
            thread::JoinHandle<u32>
        ) {

        let myaddr = self.ipaddr.clone();
        let udp_p: u16 = self.udp_port.clone();
        let (tx, rx): (mpsc::Sender<(usize, [u8; BUFFER_SIZE])>,
        mpsc::Receiver<(usize, [u8; BUFFER_SIZE])>) = mpsc::channel();

        let discover_handler_hosts = self.hosts.clone();
        let discover_handler_soc = self.socket.try_clone()
            .expect("Could not clone");
        let gateway = self.gateway;
        let discover_handler = thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(10));
                Server::start_discovery(
                    discover_handler_hosts.clone(),
                    &myaddr, udp_p,
                    discover_handler_soc.try_clone().unwrap(),
                    gateway
                    );
            }
        });

        let requests = self.requests.clone();
        let connection_num = self.connection_num.clone();
        let process_handler_hosts = self.hosts.clone();
        let process_handler_soc = self.socket.try_clone()
            .expect("Could not clone");
        let process_handler = thread::spawn(move || {
            loop {
                let (amt, data) = rx.recv().unwrap();
                let header = Server::extract_header(&data);
                let current = 16;
                let request:&str = &header.request.replace("\u{0}", "");
                println!("recived {:?}", request);
                match request {
                    "get" => {
                        Server::process_get(&data, current,
                            &header, &dir, &process_handler_soc,
                            connection_num.clone(), 
                            process_handler_hosts.clone(),
                            requests.clone(), gateway);
                    },
                    "disc" => {
                        Server::discovery(process_handler_hosts.clone(),
                        &data, 16, amt);
                    },
                    "OK" => {
                        Server::process_ok(current, data, requests.clone(),
                            header, &dir, &process_handler_soc);
                    },
                    _ => {
                        continue;
                    }
                }
            }
        });

        let listen_handler_soc = self.socket.try_clone()
            .expect("Could not clone");
        let listen_handler = thread::spawn(move || {
            loop {
                let mut buf = [32; BUFFER_SIZE];
                let (amt, _src) = listen_handler_soc.recv_from(&mut buf)
                    .expect("shit happened");
                tx.send((amt, buf)).unwrap();
            }
        });
        return (process_handler, listen_handler, discover_handler);
    }

    fn discovery(hosts: Arc<RwLock<HashMap<String, RwLock<Host>>>>,
        data: &[u8], current: usize, end: usize) {
        let mut current = current;
        let mut hosts = hosts.write().unwrap();
        while current < end { 
            let name_len = data[current];
            current += 1;
            let name = bytes::extract::extract_str(data,
                current, current + name_len as usize);
            current += name_len as usize;
            let ipaddr = bytes::extract::extract_ip(data, current);
            current += 4;
            let port = bytes::extract::extract_u16(data, current);
            current += 2;
            let key = format!("{}:{}", ipaddr, port);
            if !hosts.contains_key(&key) {
                let ip: IpAddr = ipaddr.parse().unwrap();
                let gateway = !ip.is_loopback(); 
                let host = Host::new(name.to_string(), ipaddr, port, gateway);
                hosts.insert(key, host); 
            }
        }
    }

    fn create_file_packet(
        buf: &mut [u8],
        header: &Header,
        body: &str
        ) -> usize {
            let mut current = Server::copy_header(buf, &header);
            let body_len = body.len() as u16;
            bytes::copy::copy_u16(buf, current, body_len);
            current += 2;
            bytes::copy::copy_str(buf, current, body);
            current += body_len;
            current as usize
    }

    pub fn get(
        socket: &UdpSocket,
        path: &str,
        hosts: Arc<RwLock<HashMap<String, RwLock<Host>>>>,
        src_port: u16,
        src_ip: &str,
        requests: Arc<RwLock<Vec<(String, String)>>>
    ) {
        let mut _requests = requests.write().unwrap();
        _requests.push((path.to_string(), format!("{}:{}", src_ip, src_port)));
        let clear_request = requests.clone();
        let hosts = hosts.read().unwrap();
        for (_, host) in hosts.iter() {
            let host = host.read().unwrap();
            if host.ipaddr == src_ip && host.port == src_port {
                continue;
            }
            let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
            let header = Header::new("get", host.port, src_port,
                &host.ipaddr, src_ip);
            let current = Server::create_file_packet(&mut buf, &header, path);
            Server::send(&socket, &host.ipaddr,
                host.port, buf, current as usize);
            println!("sending");
        }
        let path = path.to_string();
        let src_ip = src_ip.to_string();
        let _ = thread::spawn(move || {
            thread::sleep(Duration::from_secs(10));
            let mut req = clear_request.write().unwrap();
            let index = req.iter().position(|x| *x == 
                (path.clone(), format!("{}:{}", src_ip, src_port)));
            match index {
                Some(index) => {
                    req.remove(index);
                    println!("Could not find");
                },
                None => {
                }
            };
        });
    }

    pub fn send_discovery(
        socket: &UdpSocket,
        hosts: Arc<RwLock<HashMap<String, RwLock<Host>>>>,
        header: Header,
        gateway: bool,
        amigateway: bool
        ) {
        let mut counter = 0;
        let mut flag = true;
        let hosts = hosts.read().unwrap();
        let hosts: Vec<&RwLock<Host>> =
            hosts.iter().map(|(_, host)| host).collect();
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
                let host = hosts[i].read().unwrap();
                if !(host.ipaddr == header.src_ip &&
                    host.port == header.src_port) && 
                ((amigateway && gateway && !host.gateway) ||
                (amigateway && host.gateway && !gateway)) {
                    continue;
                }
                current = Server::copy_discovery_data(&mut buf,
                    current,
                    &host.name,
                    &host.ipaddr,
                    host.port);
            }
            Server::send(&socket, &header.dest_ip, header.dest_port,
                buf, current as usize);
        }
    }

    pub fn send(socket: &UdpSocket,
        ipaddr: &str,port: u16, buf: [u8; BUFFER_SIZE], amt: usize) {
        let ip = format!("{}:{}", ipaddr, port);
        socket.send_to(&buf[0..amt], ip)
            .expect("Could not send");
    }

    fn copy_discovery_data(buf: &mut [u8; BUFFER_SIZE],
        current: u16,
        name: &str,
        ipaddr: &str,
        port: u16) -> u16 {
        let name_len = name.len() as u8;
        let mut current = current;
        buf[current as usize] = name_len;
        current += 1;
        bytes::copy::copy_str(buf, current, name);
        current += name_len as u16;
        bytes::copy::copy_ip(buf, current, ipaddr);
        current += 4;
        bytes::copy::copy_u16(buf, current, port);
        current += 2;
        current
    }

    fn copy_header(buf: &mut [u8], header: &Header) -> u16 {
            bytes::copy::copy_str(buf, 0, &header.request);
            bytes::copy::copy_u16(buf, 4, header.dest_port);
            bytes::copy::copy_u16(buf, 6, header.src_port);
            bytes::copy::copy_ip(buf, 8, &header.dest_ip);
            bytes::copy::copy_ip(buf, 12, &header.src_ip);
            return 16;
    }

    fn extract_header(data: &[u8]) -> Header {
        let request = bytes::extract::extract_str(&data, 0, 4).trim()
            .to_string(); 
        let dest_port = bytes::extract::extract_u16(&data, 4);
        let src_port = bytes::extract::extract_u16(&data, 6);
        let dest_ip = bytes::extract::extract_ip(&data, 8);
        let src_ip = bytes::extract::extract_ip(&data, 12);
        Header {
            request,
            dest_port,
            src_port,
            dest_ip,
            src_ip
        }
    }

    fn find_file(req: &str, dir: &str) -> bool {
        let files = fs::read_dir(&dir)
            .expect("could not read dir");
        for file in files {
            let req_file = file
                .expect("Could not read from dir")
                .path();
            let req_file = req_file.to_str().unwrap();
            let file: Vec<&str> = req_file
                .split("/")
                .collect();
            let file = file.last().unwrap();
            if *file == req {
                return true;
            }
        }
        return false;
    }

    fn process_ok(
        current: usize,
        data: [u8; BUFFER_SIZE],
        requests: Arc<RwLock<Vec<(String, String)>>>,
        header: Header,
        dir: &str,
        socket: &UdpSocket
        ) {
        let mut current = current;
        let file_len = bytes::extract::extract_u16(&data,
            current) as usize;
        current += 2;
        let file = bytes::extract::extract_str(&data, current,
            current + file_len);
        current += file_len;
        let mut requests = requests.write().unwrap();
        let index = requests.iter().position(|(x, _)| *x == file);
        let dest_addr: String;
        match index {
            Some(index) => {
                let (_, b) = requests.remove(index);
                dest_addr = b;
            },
            None => {
                return;
            }
        };
        let tcp_port = bytes::extract::extract_u16(&data,
            current);
        current += 2;
        let buffer_size = bytes::extract::extract_u16(&data,
            current);
        if dest_addr != format!("{}:{}", header.dest_ip, header.dest_port) {
            let src_ip = header.dest_ip;
            let listen_tcp_port = tcp::forward(
                &header.src_ip,
                tcp_port,
                &src_ip,
                buffer_size,
                &dir,
                &file        
            );
            let splited_addr: Vec<&str> =
                dest_addr.split(":").collect();
            let dest_ip = splited_addr[0];
            let dest_port = splited_addr[1].parse::<u16>().unwrap();
            let src_port = dest_port;
            Server::send_ok(&file, listen_tcp_port, buffer_size,
                &socket, dest_port, dest_ip,
                src_port, &src_ip);
            return;
        }
        println!("buffer_size is: {}", buffer_size);
        let mut buf  = vec![0 as u8; buffer_size as usize];
        let addr = format!("{}:{}", header.src_ip, tcp_port);
        let mut tcp_connection =
            TcpStream::connect(addr).unwrap();
        let location = format!("./{}/{}", dir, file);
        let mut f = fs::File::create(location).unwrap();
        thread::spawn(move || {
            loop {
                let a = tcp_connection.read(&mut buf).unwrap(); 
                println!("read: {}", a);
                if a == 0 {
                    break;
                }
                f.write(&buf[0..a])
                    .expect("Could not write file");
            }
        });
    }

    fn increase_num_requests(hosts: Arc<RwLock<HashMap<String, RwLock<Host>>>>,
        src_ip: &str,
        src_port: u16
    ) -> u16 {
        let key = format!("{}:{}", src_ip, src_port);
        let hosts = hosts.write().unwrap();
        match hosts.get(&key) {
            Some(host) => {
                let mut host = host.write().unwrap();
                host.num_requests += 1;
                let num_requests = host.num_requests;
                return num_requests;
            },
            None => {
                return 0;
            }
        }
    }

    pub fn forward_get(
        socket: &UdpSocket,
        path: &str,
        hosts: Arc<RwLock<HashMap<String, RwLock<Host>>>>,
        src_port: u16,
        src_ip: &str,
        requests: Arc<RwLock<Vec<(String, String)>>>,
        blocked_host: &str 
    ) {
        let mut _requests = requests.write().unwrap();
        _requests.push((path.to_string(), blocked_host.to_string()));
        let clear_request = requests.clone();
        let hosts = hosts.read().unwrap();
        for (_, host) in hosts.iter() {
            let host = host.read().unwrap();
            let host_addr = format!("{}:{}", host.ipaddr, host.port);
            if host_addr == blocked_host {
                continue;
            }
            if host.ipaddr == src_ip && host.port == src_port {
                continue;
            }
            let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
            let header = Header::new("get", host.port, src_port,
                &host.ipaddr, src_ip);
            let current = Server::create_file_packet(&mut buf, &header, path);
            Server::send(&socket, &host.ipaddr,
                host.port, buf, current as usize);
            println!("sending");
        }
        let path = path.to_string();
        let blocked_host = blocked_host.to_string();
        let _ = thread::spawn(move || {
            thread::sleep(Duration::from_secs(10));
            let mut req = clear_request.write().unwrap();
            let index = req.iter().position(|x| *x 
                == (path.clone(), blocked_host.clone()));
            match index {
                Some(index) => {
                    req.remove(index);
                    println!("Could not find");
                },
                None => {
                }
            };
        });
    }

    fn process_get(
        data: &[u8],
        current: usize,
        header: &Header,
        dir: &str,
        socket: &UdpSocket,
        connection_num: Arc<RwLock<u16>>, 
        hosts: Arc<RwLock<HashMap<String, RwLock<Host>>>>,
        requests: Arc<RwLock<Vec<(String, String)>>>,
        amigateway: bool
    ) {
        let mut current = current;
        let num_requests: u16 = Server::increase_num_requests(hosts.clone(),
        &header.src_ip, header.src_port);
        if num_requests == 0 {
            return;
        }
        let req_file_len = bytes::extract::extract_u16(&data, current);
        current += 2;
        let req_file = bytes::extract::extract_str(&data,
            current, current + req_file_len as usize);
        if Server::find_file(req_file, &dir) {
            let addr = format!("{}:{}", header.dest_ip, 0);
            let listener = TcpListener::bind(addr).unwrap();
            let socket_addr = listener.local_addr().unwrap();
            let port = socket_addr.port();
            let file = req_file.to_string();
            let directory = dir.to_string();
            let buffer_size: u16; 
            {
                let mut num = connection_num.write().unwrap();
                if *num > MAX_CONEECTION  {
                    return;
                }
                *num += 1;
                buffer_size = Server::calculate_buffer(*num, num_requests);
            }
            thread::spawn(move || {
                match listener.accept() {
                    Ok((mut socket, _addr)) => {
                        socket.set_nodelay(true)
                            .expect("Could not set no delay");
                        let mut buffer 
                            = vec![0 as u8; buffer_size as usize];
                        let location = format!("./{}/{}", directory, file);
                        let mut f = fs::File::open(location)
                            .expect("Could not open file");
                        loop {
                            let a = f.read(&mut buffer).unwrap(); 
                            println!("wrote: {}", a);
                            if a == 0 {
                                break;
                            }
                            socket.write(&buffer[0..a]).unwrap();
                        }
                    },
                    Err(e) => println!("couldn't get client: {:?}", e)
                }
                let mut num = connection_num.write().unwrap();
                *num -= 1;
            });
            Server::send_ok(req_file, port, buffer_size, socket,
                header.src_port, &header.src_ip, header.dest_port,
                &header.dest_ip);
            return;
        }
        if amigateway {
            let blocked_host =
                format!("{}:{}", header.src_ip, header.src_port);
            Server::forward_get(
                socket.clone(),
                &req_file,
                hosts.clone(),
                header.dest_port,
                &header.dest_ip,
                requests.clone(),
                &blocked_host
            );
        }
    }

    fn start_discovery(
        hosts: Arc<RwLock<HashMap<String, RwLock<Host>>>>, 
        myaddr: &str,
        udp_p: u16,
        socket: UdpSocket,
        gateway: bool
    ) {
        let _hosts = hosts.read().unwrap();
        for (_, host) in _hosts.iter() {
            let host = host.read().unwrap();
            if host.ipaddr == myaddr && host.port == udp_p {
                continue
            }
            let header = Header::new("disc",
                host.port, udp_p, &host.ipaddr, &myaddr);
            Server::send_discovery(
                &socket,
                hosts.clone(),
                header,
                host.gateway,
                gateway
            );
        }
    }

    fn send_ok(
        req_file: &str,
        port: u16,
        buffer_size: u16,
        socket: &UdpSocket,
        dest_port: u16,
        dest_ip: &str,
        src_port: u16,
        src_ip: &str
        ) {
        let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        let resph = Header::new("OK", dest_port, src_port, dest_ip, src_ip);
        let mut current 
            = Server::create_file_packet(&mut buf,
            &resph, req_file);
        bytes::copy::copy_u16(&mut buf, current as u16, port);
        current += 2;
        bytes::copy::copy_u16(&mut buf, current as u16, buffer_size);
        current += 2;
        Server::send(&socket, dest_ip,
            dest_port, buf, current);
    }

    pub fn calculate_buffer(cons_num: u16, num_requests: u16) -> u16 {
        BUFFER_SIZE as u16 / (cons_num + num_requests / 4)
    }

}
