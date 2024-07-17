use anyhow::Result;
use clap::Parser;

use crate::command::ArgsParser;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    id: usize,
}

pub fn upload(args: &[&str]) -> Result<()> {
    let args = Args::parse_args("upload", args)?;
    Ok(())
}
