use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;

use std::thread;
use std::fs;
use std::sync::mpsc::{Sender, Receiver, channel};

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

