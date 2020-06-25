extern crate getopts;
use std::env;
use getopts::Options;
use std::path::Path;

mod server;
mod bytes;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn add_options() -> Options {
    let mut opts = Options::new();
    opts.optflag("h", "help", "This is help menu");
    opts.optopt("p", "port", "Enter a port for udp server", "PORT");
    opts.optopt("i", "ip", "Enter an ip to listen on", "IP");
    opts.optopt("l", "list", "List of hosts", "LIST");
    opts.optopt("d", "dir", "Files Directory", "DIR");
    opts
}

fn parse_arg(opts: Options) -> (String, String, String, String) {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m },
        Err(_f) => { 
            print_usage(&program, opts);
            std::process::exit(1);
        }
    };

    // mathcing help
    if matches.opt_present("h") {
        print_usage(&program, opts);
        std::process::exit(0);
    }    

    // mathcing port
    let port = matches.opt_str("p");
    let port = match port {
        Some(x) => x,
        None => {
            eprintln!("Please enter a valid port");
            print_usage(&program, opts);
            std::process::exit(0);
        }
    };

    // matching list of hosts
    let location = matches.opt_str("l");
    let location = match location {
        Some(x) => x,
        None => {
            "".to_string()
        }
    };
    let path = Path::new(&location);
    if !path.exists() { 
        eprintln!("Please enter a valid location for hosts list");
        print_usage(&program, opts);
        std::process::exit(0);
    }

    // mathcing files directory
    let dir_location = matches.opt_str("d");
    let dir_location = match dir_location {
        Some(x) => x,
        None => {
            "".to_string()
        }
    };
    // checking if the file exists
    let path = Path::new(&dir_location);
    if !path.exists() { 
        eprintln!("Please enter a valid location for files");
        print_usage(&program, opts);
        std::process::exit(0);
    }
    // matching ip
    let ip = matches.opt_str("i");
    let ip = match ip {
        Some(x) => x,
        None => {
            "127.0.0.1".to_string()
        }
    };
    return (port, ip, location, dir_location);
}

fn main() {
    let opts = add_options();
    let (port, ip, location, dir_location) = parse_arg(opts);
    println!("listening on {:?}:{}", ip, port);
    println!("list of hosts are provided at: {:?}", location);
    println!("servicing files of: {:?}", dir_location);
    server::start(port, location, dir_location, ip);
}
