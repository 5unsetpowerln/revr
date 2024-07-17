use anyhow::Result;
use clap::Parser;

use crate::command::ArgsParser;

#[derive(Parser)]
struct Args {
    id: usize,
}

pub async fn back(args: &[&str]) -> Result<()> {
    Args::parse_args("back", args)?;
    super::sessions::sessions(args).await
}
