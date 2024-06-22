pub mod shell;

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

static SESSIONS: Lazy<Mutex<Vec<Session>>> = Lazy::new(|| Mutex::new(vec![]));
static NEXT_ID: Lazy<Mutex<usize>> = Lazy::new(|| Mutex::new(0));

#[derive(Debug)]
struct Session {
    tcp_stream: TcpStream,
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