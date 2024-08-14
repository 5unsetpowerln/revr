use anyhow::{bail, Result};
use clap::Parser;
use log::info;
use tokio::select;

use crate::shell::Shell;

use super::Cmd;

#[derive(Parser)]
pub struct Args {
    id: Option<usize>,
}

impl Cmd for Args {
    async fn run(self, rl: &mut Shell) -> Result<()> {
        if self.id.is_none() {
            use prettytable::{row, Table};

            let sessions = crate::session::get_sessions();
            let mut table = Table::new();

            table.add_row(row!["id", "address"]);
            for session in sessions {
                table.add_row(row![
                    session.id.to_string(),
                    session.remote_addr.to_string()
                ]);
            }

            println!("{}", table);
            return Ok(());
        }

        let id = self.id.unwrap();
        match crate::session::shell::start(id).await.unwrap() {
            crate::session::shell::ShellMessage::Closed => {
                println!();
                info!("session {} is closed", id);
            }
            crate::session::shell::ShellMessage::Paused => {
                println!();
                info!("session {} is paused", id);
            }
        };

        Ok(())
    }
}
