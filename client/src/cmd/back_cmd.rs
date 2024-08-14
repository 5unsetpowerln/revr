use anyhow::Result;
use clap::Parser;

use crate::shell::Shell;

use super::Cmd;

#[derive(Parser)]
pub struct Args {
    id: usize,
}

// pub async fn back(args: &[&str], app_state: &mut LocalState) -> Result<()> {
// Args::parse_args("back", args)?;
// super::sessions::sessions(args, app_state).await
// }

// impl Cmd for Args {
// fn run(self, rl: &mut Shell) -> Result<()> {

// }
// }
