use std::{
    io::{BufReader, Read, Write},
    net::{IpAddr, SocketAddr, TcpListener, TcpStream},
    str::FromStr,
    sync::Mutex,
};

use anyhow::{anyhow, bail, Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use once_cell::sync::Lazy;
use tokio::{select, sync::watch, task::JoinHandle};

pub static SESSIONS: Lazy<Mutex<Vec<Session>>> = Lazy::new(|| Mutex::new(vec![]));
static NEXT_ID: Lazy<Mutex<usize>> = Lazy::new(|| Mutex::new(0));

#[derive(Debug)]
pub struct Session {
    pub tcp_stream: TcpStream,
    pub metadata: Metadata,
}

#[derive(Debug, Clone)]
pub struct Metadata {
    pub remote_addr: SocketAddr,
    pub id: usize,
}

impl Session {
    fn new(port: u16) -> Result<Self> {
        let addr = {
            let ip = IpAddr::from_str("127.0.0.1")?;
            SocketAddr::new(ip, port)
        };

        let tcp_listener = TcpListener::bind(addr)?;
        let (tcp_stream, remote_addr) = tcp_listener.accept()?;

        let id = {
            let mut next_id = NEXT_ID.lock().unwrap();
            let i = *next_id;
            *next_id += 1;
            i
        };

        Ok(Self {
            tcp_stream,
            metadata: Metadata { remote_addr, id },
        })
    }
}

pub fn create(port: u16) -> Result<()> {
    let new_session = Session::new(port).context("failed to create session")?;

    {
        let mut sessions = SESSIONS.lock().unwrap();
        sessions.push(new_session);
    }

    Ok(())
}

pub fn get_sessions() -> Vec<Metadata> {
    let sessions = SESSIONS.lock().unwrap();
    sessions.iter().map(|s| s.metadata.clone()).collect()
}

pub async fn stdout_stream_pipe(stream: TcpStream, recver: watch::Receiver<()>) -> JoinHandle<()> {
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
                    break
                }

                result = read(&mut reader, &mut buffer) => {
                match result
                 {
                    Ok(0) => {
                        break;
                    }
                    Ok(n) => {
                        std::io::stdout().write_all(&buffer[..n]).unwrap();
                        std::io::stdout().flush().unwrap();
                    }
                    Err(e) => {
                        println!("{}", e);
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
) -> JoinHandle<Result<&'static str>> {
    tokio::spawn(async move {
        let mut writer = stream;
        enable_raw_mode()?;

        'pipe_loop: loop {
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
                                sender.send(())?;
                                break 'pipe_loop;
                            }
                            writer.write_all(&key_sequence)?;
                        }
                        KeyCode::Enter => writer.write_all(b"\n")?,
                        KeyCode::Backspace => writer.write_all(b"\x08")?,
                        KeyCode::Esc => writer.write_all(b"\x1b")?,
                        _ => {}
                    }
                }
            }
        }
        disable_raw_mode()?;
        Ok("")
    })
}

pub async fn start(id: usize) -> Result<()> {
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
    // let mut writer = session.tcp_stream.try_clone().unwrap();
    let (sender, recver) = watch::channel(());

    let t1 = stdout_stream_pipe(session.tcp_stream.try_clone().unwrap(), recver).await;
    let t2 = stdin_stream_pipe(session.tcp_stream.try_clone().unwrap(), sender).await;
    t1.await?;
    t2.await?;
    // println!("hello");

    let session = Session {
        tcp_stream: saved_tcp_stream,
        metadata: session.metadata,
    };

    let mut sessions = SESSIONS.lock().unwrap();
    sessions.push(session);

    Ok(())
}