use anyhow::Result;
use clap::Parser;

use crate::{command::ArgsParser, LocalState};

#[derive(Parser)]
struct Args {
    id: usize,
}

pub async fn back(args: &[&str], app_state: &mut LocalState) -> Result<()> {
    Args::parse_args("back", args)?;
    super::sessions::sessions(args, app_state).await
}
