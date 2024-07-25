use anyhow::{bail, Result};
use clap::Parser;
use log::info;
use tokio::select;

use crate::{command::ArgsParser, LocalState};

#[derive(Parser)]
struct Args {
    id: Option<usize>,
}

pub async fn sessions(args: &[&str], local_state: &mut LocalState) -> Result<()> {
    // let mut stdio_proxy = StdioProxy::new(false)?;
    // // stdio_proxy.wait_until(3)

    // select! {
    //     _ = stdio_proxy.wait_until(3) => {
    //         stdio_proxy.close();
    //         bail!("interrupted!")
    //     }
    //     result = handle(args, app_state) => {
    //         stdio_proxy.close();
    //         return result
    //     }
    // }
    let args = Args::parse_args("sessions", args)?;

    if args.id.is_none() {
        use prettytable::{row, Table};

        let sessions = super::session::get_sessions();
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
    local_state.session_ctx = Some(id);
    match super::session::shell::start(id).await.unwrap() {
        super::session::shell::ShellMessage::Closed => {
            println!();
            info!("session {} is closed", id);
        }
        super::session::shell::ShellMessage::Paused => {
            println!();
            info!("session {} is paused", id);
        }
    }

    Ok(())
}

async fn handle(args: &[&str], local_state: &mut LocalState) -> Result<()> {
    Ok(())
}
