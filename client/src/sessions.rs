use anyhow::Result;
use clap::Parser;
use log::info;

use crate::command::ArgsParser;

#[derive(Parser)]
struct Args {
    id: Option<usize>,
}

pub async fn sessions(args: &[&str]) -> Result<()> {
    let args = Args::parse_args("sessions", args)?;

    if args.id.is_none() {
        use prettytable::{row, Table};

        let sessions = super::revshell::get_sessions();
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

    let id = args.id.unwrap();
    match super::revshell::shell::start(id).await.unwrap() {
        super::revshell::shell::ShellMessage::Closed => {
            println!();
            info!("session {} is closed", id);
        }
        super::revshell::shell::ShellMessage::Paused => {
            println!();
            info!("session {} is paused", id);
        }
    }

    Ok(())
}
