use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;

use std::thread;
use std::fs;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::{Sender, Receiver, channel};

pub fn write_to_client(
    ip: &str,
    req_file: &str,
    dir: &str,
    buffer_size: u16,
    connection_num: Arc<RwLock<u16>>, 
) -> u16 {
    let addr = format!("{}:{}", ip, 0);
    let listener = TcpListener::bind(addr).unwrap();
    let socket_addr = listener.local_addr().unwrap();
    let port = socket_addr.port();
    let file = req_file.to_string();
    let directory = dir.to_string();
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
    port
}

pub fn read_from_server(
    buffer_size: u16,
    ip: &str,
    port: u16,
    dir: &str,
    file: &str
) {
    let mut buf  = vec![0 as u8; buffer_size as usize];
    let addr = format!("{}:{}", ip, port);
    let mut tcp_connection =
        TcpStream::connect(addr).unwrap();
    let location = format!("./{}/{}", dir, file);
    let mut f = fs::File::create(location).unwrap();
    thread::spawn(move || {
        loop {
            let a = tcp_connection.read(&mut buf).unwrap(); 
            if a == 0 {
                break;
            }
            f.write(&buf[0..a])
                .expect("Could not write file");
        }
    });
}

pub fn forward(
    src_ip: &str,
    src_port: u16,
    self_ip: &str,
    buffer_size: u16,
    dir: &str,
    file: &str,
) -> u16 {
    let mut buf  = vec![0 as u8; buffer_size as usize];
    let addr = format!("{}:{}", src_ip, src_port);
    let (tx, rx): (Sender<(Vec<u8>, usize)>,
    Receiver<(Vec<u8>, usize)>) = channel();
    let mut tcp_connection =
        TcpStream::connect(addr).unwrap();
    let location = format!("./{}/{}", dir, file);
    let mut f = fs::File::create(location).unwrap();
    thread::spawn(move || {
        loop  {
            let count = tcp_connection.read(&mut buf).unwrap();
            if count <= 0 {
                break;
            }
            f.write(&mut buf[0..count])
                .expect("Could not write file");
            tx.send((buf.clone(), count)).unwrap();
        }
        tx.send((buf, 0)).unwrap();
    });
    let addr = format!("{}:{}", self_ip, 0);
    let listener = TcpListener::bind(addr).unwrap();
    let socket_addr = listener.local_addr().unwrap();
    let port = socket_addr.port();
    thread::spawn(move || {
        match listener.accept() {
            Ok((mut socket, _addr)) => {
                socket.set_nodelay(true)
                    .expect("Could not set no delay");
                loop {
                    let (data, count) = rx.recv().unwrap();
                    if count == 0 {
                        break;
                    }
                    socket.write(&data[0..count]).unwrap();
                }
            },
            Err(e) => println!("couldn't get client: {:?}", e)
        }
    });
    port
}
