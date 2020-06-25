use std::sync::{RwLock};


pub struct Host {
    pub name: String,
    pub ipaddr: String,
    pub port: u16,
    pub num_requests: u16,
    pub gateway: bool
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
