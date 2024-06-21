use std::{
    io::{BufReader, BufWriter},
    net::{IpAddr, SocketAddr, TcpListener, TcpStream},
    str::FromStr,
    sync::Mutex,
};

use anyhow::{anyhow, bail, Context, Result};
use once_cell::sync::Lazy;

static SESSIONS: Lazy<Mutex<Vec<Session>>> = Lazy::new(|| Mutex::new(vec![]));

#[derive(Debug)]
struct Session {
    writer: BufWriter<TcpStream>,
    reader: BufReader<TcpStream>,
    metadata: SessionMetadata,
}

#[derive(Debug, Clone)]
pub struct SessionMetadata {
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
        let reader = BufReader::new(tcp_stream.try_clone()?);
        let writer = BufWriter::new(tcp_stream);

        let id = {
            let sessions = SESSIONS.lock().unwrap();
            if sessions.len() == 0 {
                0
            } else {
                sessions.last().unwrap().metadata.id + 1
            }
        };

        let metadata = SessionMetadata::new(remote_addr, id);

        Ok(Self {
            writer,
            reader,
            metadata,
        })
    }
}

impl SessionMetadata {
    fn new(remote_addr: SocketAddr, id: usize) -> Self {
        Self { remote_addr, id }
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

pub fn get_metadatas() -> Vec<SessionMetadata> {
    let metadatas = {
        let sessions = SESSIONS.lock().unwrap();
        let metadatas = sessions.iter().map(|x| x.metadata.clone()).collect();
        metadatas
    };

    metadatas
}

pub fn stdio_forward(id: usize) -> Result<()> {
    let sessions = SESSIONS.lock().unwrap();
    let mut session = if let Some(s) = sessions.iter().find(|x| x.metadata.id == id) {
        s
    } else {
        bail!(anyhow!("session with id {} was not found", id));
    };
    
    

    Ok(())
}
