mod udp_server;

pub fn start() {
    println!("Hello, world!");
    let connection = udp_server::UdpCon::init();
    loop {
        let (amt, src, buf) = connection.listen();
        let s = std::str::from_utf8(&buf[..amt])
            .expect("Something happened while converting from utf8 to string!!");
        println!("recived {:?}", s);
    }
}
