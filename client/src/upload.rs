use anyhow::Result;
use clap::Parser;

use crate::{command::ArgsParser, LocalState};

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    id: usize,
}

pub async fn upload(args: &[&str], app_state: &mut LocalState) -> Result<()> {
    let args = Args::parse_args("upload", args)?;
    Ok(())
}
