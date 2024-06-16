use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener},
    str::FromStr,
};

struct Session {}

impl Session {
    fn new(port: u16) -> Self {
        let addr = {
            let ip = IpAddr::from_str("127.0.0.1").unwrap();
            SocketAddr::new(ip, port)
        };

        let tcp_listener = TcpListener::bind(addr).unwrap();
        let (tcp_stream, remote_addr) = tcp_listener.accept().unwrap();

        Self {}
    }
}

pub mod commands {
    use clap::Parser;

    //
    // listen command
    //
    pub fn help() {
        println!("listen --port [port]");
    }

    pub fn listen(args: &str) {
        #[derive(Parser, Debug)]
        struct Args {
            #[arg(short, long)]
            port: u16,
        }

        let args = if let Ok(a) = Args::try_parse_from(args.split_whitespace()) {
            a
        } else {
            help();
            return;
        };
    }
}

//
// session command
//
// #[derive(Parser, Debug)]
// struct
