use anyhow::{anyhow, bail, Result};
use mio::unix::SourceFd;
use mio::{Events, Interest, Poll, Token};
use std::{
    io::{self, BufReader, Read, Write},
    net::TcpStream,
    os::fd::AsRawFd,
};
use termion::raw::IntoRawMode;
use tokio::{select, sync::watch, task::JoinHandle};

use crate::cli;
use crate::cli::ascii::char_to_ctrl;

use super::{Session, SESSIONS};

async fn stdout_stream_pipe(
    stream: TcpStream,
    recver: watch::Receiver<()>,
) -> JoinHandle<Result<ShellMessage>> {
    tokio::spawn(async move {
        let mut buffer = [0; 1024];
        let mut recver = recver;
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(1)))
            .unwrap();
        let mut reader = BufReader::new(stream);

        async fn read(reader: &mut BufReader<TcpStream>, buffer: &mut [u8]) -> Result<usize> {
            let len = reader.read(buffer)?;
            Ok(len)
        }

        loop {
            select! {
                _ = recver.changed() => {
                    return Ok(ShellMessage::Paused);
                }

                result = read(&mut reader, &mut buffer) => {
                match result
                 {
                    Ok(0) => {
                        return Ok(ShellMessage::Closed)
                    }
                    Ok(n) => {
                        std::io::stdout().write_all(&buffer[..n]).unwrap();
                        std::io::stdout().flush().unwrap();
                    }
                    Err(_e) => {
                        continue;
                    }
                }}
            }
        }
    })
}

async fn stdin_stream_pipe(
    stream: TcpStream,
    sender: watch::Sender<()>,
) -> JoinHandle<Result<ShellMessage>> {
    tokio::spawn(async move {
        let mut writer = stream;

        let _stdout = io::stdout().into_raw_mode().unwrap();
        let mut stdin = io::stdin().lock();
        let fd = stdin.as_raw_fd();

        // making poll of mio
        let mut poll = Poll::new().unwrap();
        let mut events = Events::with_capacity(1024);

        // register the fd of stdin
        poll.registry()
            .register(
                &mut SourceFd(&fd),
                Token(0),
                Interest::READABLE | Interest::WRITABLE,
            )
            .unwrap();

        fn send(writer: &mut TcpStream, data: &[u8]) -> Result<()> {
            let size = data.len().to_le_bytes();
            let mut header = [0; 11];
            header[0] = 0xff;
            header[1] = 0x01;
            header[2..10].copy_from_slice(&size[..8]);
            header[10] = 0xff;
            writer.write_all(&header)?;
            writer.write_all(data)?;
            Ok(())
        }

        let mut received_byte = [0; 1];
        let mut buffer = Vec::new();
        let exit_ctrl_char = char_to_ctrl(b'D')?;
        let example_keybind_to_switch_buffering = char_to_ctrl(b'I')?;
        // let example_keybind_to_stop_buffering_input = char_to_ctrl(b'I')?;
        let mut is_buffering = false;

        loop {
            // waiting for event
            poll.poll(&mut events, None).unwrap();

            for event in &events {
                if event.token() == Token(0) && event.is_readable() {
                    // stdinからバイトを読み取る
                    match stdin.read(&mut received_byte) {
                        Ok(1) => {
                            {
                                if received_byte[0] == exit_ctrl_char {
                                    sender.send(())?;
                                    return Ok(ShellMessage::Paused);
                                }
                                if received_byte[0] == example_keybind_to_switch_buffering {
                                    is_buffering = !is_buffering;
                                }
                            }

                            // println!("{}", is_buffering);
                            if is_buffering && received_byte[0] != b'\r' {
                                if received_byte[0] != example_keybind_to_switch_buffering {
                                    buffer.push(received_byte[0]);
                                }
                                continue;
                            } else if received_byte[0] != example_keybind_to_switch_buffering {
                                buffer.push(received_byte[0]);
                            }

                            send(&mut writer, &buffer)?;
                            buffer.clear();
                        }
                        Ok(_) => {
                            panic!("recieved EOF from stdin")
                        }
                        Err(e) => {
                            // エラー処理
                            eprintln!("Error reading stdin: {}", e);
                            break;
                        }
                    }
                }
            }
        }
    })
}

pub async fn start(id: usize) -> Result<ShellMessage> {
    let session = {
        let mut sessions = SESSIONS.lock().unwrap();

        let mut index = None;
        for (i, s) in sessions.iter().enumerate() {
            if s.metadata.id == id {
                index = Some(i);
            }
        }
        if index.is_none() {
            bail!(anyhow!("session with id {} was not found", id));
        }

        sessions.remove(index.unwrap())
    };

    let saved_tcp_stream = session.tcp_stream.try_clone().unwrap();
    let (sender, recver) = watch::channel(());

    let t1 = stdout_stream_pipe(session.tcp_stream.try_clone().unwrap(), recver).await;
    let t2 = stdin_stream_pipe(session.tcp_stream.try_clone().unwrap(), sender).await;
    let msg1 = t1.await??;
    let msg2 = t2.await??;

    match msg1 {
        ShellMessage::Closed => {
            return Ok(ShellMessage::Closed);
        }
        ShellMessage::Paused => match msg2 {
            ShellMessage::Closed => return Ok(ShellMessage::Closed),
            ShellMessage::Paused => (),
        },
    }

    let session = Session {
        tcp_stream: saved_tcp_stream,
        metadata: session.metadata,
    };

    let mut sessions = SESSIONS.lock().unwrap();
    sessions.push(session);

    Ok(ShellMessage::Paused)
}

pub enum ShellMessage {
    Closed,
    Paused,
}
