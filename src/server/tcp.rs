use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;
use std::thread;
use std::fs;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::sync::{Arc, RwLock};
use crate::server::udp;

pub fn forward(src_ip: &str,
    src_port: u16,
    self_ip: &str,
    buffer_size: u16,
    dir: &str,
    file: &str,
    connection_num: Arc<RwLock<u16>>) -> u16 {
    let mut buf  = vec![0 as u8; buffer_size as usize];
    let addr = format!("{}:{}", src_ip, src_port);
    let (tx, rx): (Sender<(Vec<u8>, bool)>,
    Receiver<(Vec<u8>, bool)>) = channel();
    let mut tcp_connection =
        TcpStream::connect(addr).unwrap();
    let location = format!("./{}/{}", dir, file);
    let mut f = fs::File::create(location).unwrap();
    thread::spawn(move || {
        while tcp_connection.read(&mut buf).unwrap() != 0 {
            f.write(&mut buf)
                .expect("Could not write file");
            tx.send((buf.clone(), true)).unwrap();
        }
        tx.send((buf, false)).unwrap();
    });
    let addr = format!("{}:{}", self_ip, 0);
    let listener = TcpListener::bind(addr).unwrap();
    let socket_addr = listener.local_addr().unwrap();
    let port = socket_addr.port();
    {
        let mut num = connection_num.write().unwrap();
        if *num > udp::MAX_CONEECTION  {
            return 0;
        }
        *num += 1;
    }
    thread::spawn(move || {
        match listener.accept() {
            Ok((mut socket, _addr)) => {
                socket.set_nodelay(true)
                    .expect("Could not set no delay");
                loop {
                    let (data, flag) = rx.recv().unwrap();
                    if !flag {
                        break;
                    }
                    socket.write(&data).unwrap();
                }
            },
            Err(e) => println!("couldn't get client: {:?}", e)
        }
        let mut num = connection_num.write().unwrap();
        *num -= 1;
    });
    port
}

