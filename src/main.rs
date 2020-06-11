extern crate getopts;
use std::env;
use getopts::Options;

mod server;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn parse_arg() -> String {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "This is help menu");
    opts.optopt("p", "port", "Enter a port for udp server", "PORT");
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
    match port {
        Some(x) => x,
        None => {
            eprintln!("Please enter a valid port");
            print_usage(&program, opts);
            std::process::exit(0);
        }
    }
}

fn main() {
    let port = parse_arg();
    println!("{:?}", port);
    server::start(port);
}
