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

fn parse_arg() -> (String, String, String) {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "This is help menu");
    opts.optopt("p", "port", "Enter a port for udp server", "PORT");
    opts.optopt("l", "list", "List of hosts", "LIST");
    opts.optopt("d", "dir", "Files Directory", "DIR");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m },
        Err(_f) => { 
            print_usage(&program, opts);
            std::process::exit(1);
        }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        std::process::exit(0);
    }    
    let port = matches.opt_str("p");
    let port = match port {
        Some(x) => x,
        None => {
            eprintln!("Please enter a valid port");
            print_usage(&program, opts);
            std::process::exit(0);
        }
    };
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
    let dir_location = matches.opt_str("d");
    let dir_location = match dir_location {
        Some(x) => x,
        None => {
            "".to_string()
        }
    };
    let path = Path::new(&dir_location);
    if !path.exists() { 
        eprintln!("Please enter a valid location for files");
        print_usage(&program, opts);
        std::process::exit(0);
    }
    return (port, location, dir_location);
}

fn main() {
    let (port, location, dir_location) = parse_arg();
    println!("{:?}", port);
    println!("{:?}", location);
    println!("{:?}", dir_location);
    server::start(port, location, dir_location);
}
