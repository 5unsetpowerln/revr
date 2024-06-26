use std::{
    io::{BufReader, Read, Write},
    net::TcpStream,
};

use anyhow::{anyhow, bail, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use tokio::{select, sync::watch, task::JoinHandle};

use super::{Session, SESSIONS};

pub async fn stdout_stream_pipe(
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

pub async fn stdin_stream_pipe(
    stream: TcpStream,
    sender: watch::Sender<()>,
) -> JoinHandle<Result<ShellMessage>> {
    tokio::spawn(async move {
        let mut writer = stream;
        enable_raw_mode()?;

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

        loop {
            if event::poll(std::time::Duration::from_millis(500))? {
                if let Event::Key(KeyEvent {
                    code,
                    modifiers,
                    kind,
                    state,
                }) = event::read()?
                {
                    match code {
                        KeyCode::Char(c) => {
                            let mut key_sequence = vec![c as u8];
                            if modifiers.contains(KeyModifiers::CONTROL) {
                                key_sequence.insert(0, 0x1b);
                            }
                            if key_sequence == vec![0x1b, 100] {
                                disable_raw_mode()?;
                                sender.send(())?;
                                return Ok(ShellMessage::Paused);
                            }
                            send(&mut writer, &key_sequence)?;
                        }
                        KeyCode::Enter => send(&mut writer, b"\n")?,
                        KeyCode::Backspace => send(&mut writer, b"\x08")?,
                        KeyCode::Esc => send(&mut writer, b"\x1b")?,
                        _ => {}
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
