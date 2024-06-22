use anyhow::{Context, Result};
use clap::Parser;
use log::info;

use crate::{command::ArgsParser, revshell};

#[derive(Parser, Debug)]
#[command(name = "listen")]
struct Args {
    port: u16,
}

pub fn listen(args: &[&str]) -> Result<()> {
    let args = Args::parse_args("listen", args)?;

    info!("waiting for reverse shell on port {}", args.port);
    revshell::create(args.port).context("failed to create new session")?;
    info!("connection established!");

    Ok(())
}
