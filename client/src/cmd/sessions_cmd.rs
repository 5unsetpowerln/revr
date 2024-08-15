use anyhow::{Context, Result};
use clap::Parser;
use log::{debug, info};
use tokio::{
    select,
    sync::watch::{self, Receiver},
    time::error::Elapsed,
};

use crate::{
    session::shell::ShellMessage,
    shell::{Shell, SignalRegister, SignalRegisterPair},
};

use super::Cmd;

#[derive(Parser)]
pub struct Args {
    /// Reverse shell session with specified id will be used.
    /// If you don't specify any id, all the sessions will be listed.
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

        // let signal_register_pair_for_reverse_shell_pipe = rl.signal_register();
        // let is_reverse_shell_pipe_running = Arc::new(Mutex::new(false));
        let (tx, mut rx) = watch::channel(1);
        let reverse_shell_pipe = tokio::spawn(async move {
            debug!("starting reverse-shell-pipe");

            let result = crate::session::shell::start(id).await;
            tx.send(0).unwrap();

            debug!("finished reverse-shell-pipe");

            return result;
        });

        let signal_register_pair_for_signal_canceler = rl.signal_register();
        let signal_canceler = tokio::spawn(async move {
            debug!("{}", "started signal_canceler");
            let sig_reg_pair = signal_register_pair_for_signal_canceler;

            loop {
                let sig_reg_locked = sig_reg_pair.0.lock().unwrap();
                sig_reg_locked.catch_ctrl();
                let sig_reg_locked = sig_reg_pair.1.wait(sig_reg_locked).unwrap();
                match rx.has_changed() {
                    Ok(has_changed) => {
                        if has_changed {
                            sig_reg_locked.catch_ctrl();
                            debug!("finishing signal_canceler");
                            break;
                        } else {
                            info!("Please wait");
                            continue;
                        }
                    }
                    Err(_) => {
                        sig_reg_locked.catch_ctrl();
                        debug!("finishing signal_canceler");
                        break;
                    }
                }
            }
        });

        select! {
            join_result = reverse_shell_pipe => {
                let shell_msg = join_result.context("Failed to join reverse-shell-pipe")?.context("Failed to finish reverse-shell-pipe properly")?;
                match shell_msg {
                    ShellMessage::Closed => {
                        println!();
                        info!("session {} is closed", id);
                    }
                    ShellMessage::Paused => {
                        println!();
                        info!("session {} is paused", id);
                    }
                }

                let sig_reg_pair = rl.signal_register();
                sig_reg_pair.1.notify_all();
            }
            _ = signal_canceler => {}
        }

        Ok(())
    }
}
