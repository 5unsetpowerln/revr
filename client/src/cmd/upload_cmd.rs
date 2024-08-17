use anyhow::Result;
use clap::Parser;

use crate::shell::Shell;

use super::Cmd;

#[derive(Parser)]
pub struct Args {
    /// Reverse shell session with specified id will start.
    /// If you don't specify an id, all the sessions will be listed and showed.
    id: Option<usize>,
}

impl Cmd for Args {
    async fn run(self, rl: &mut Shell) -> Result<()> {
        Ok(())
    }
}
