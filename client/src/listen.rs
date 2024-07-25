use anyhow::{Context, Result};
use clap::Parser;
use log::info;

use crate::{command::ArgsParser, session, LocalState};

#[derive(Parser, Debug)]
#[command(name = "listen")]
struct Args {
    port: u16,
}

pub async fn listen(args: &[&str], app_state: &mut LocalState) -> Result<()> {
    let args = Args::parse_args("listen", args)?;

    info!("waiting for reverse shell on port {}", args.port);
    session::create(args.port).context("failed to create new session")?;
    info!("connection established!");

    Ok(())
}
