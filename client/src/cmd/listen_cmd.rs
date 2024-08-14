use anyhow::{Context, Result};
use clap::Parser;
use log::info;

use crate::{session, shell::Shell};

use super::Cmd;

// use crate::{command::ArgsParser, session, LocalState};

#[derive(Parser, Debug)]
pub struct Args {
    port: u16,
}

impl Cmd for Args {
    async fn run(self, rl: &mut Shell) -> Result<()> {
        info!("waiting for reverse shell on port {}", self.port);
        session::create(self.port).context("failed to create new session")?;
        info!("connection established!");
        Ok(())
    }
}
