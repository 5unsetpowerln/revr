use anyhow::{Context, Result};
use clap::Parser;
use log::{debug, info};
use tokio::{select, sync::watch};

use crate::{session::shell::ShellMessage, shell::Shell};

use super::Cmd;

#[derive(Parser)]
pub struct Args {
    /// Reverse shell session with specified id will start.
    /// If you don't specify an id, all the sessions will be listed and showed.
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

        // This channel is used to tell the end of remote shell pipe.
        let (endof_remote_shell_tx, endof_remote_shell_rx) = watch::channel(());

        // This tusk runs the remote-shell-pipe. When remote-shell-pipe is finished, this tell the end of remote-shell-pipe to
        // the interrupt_ignoring task to let it finish.
        let reverse_shell_pipe = tokio::spawn(async move {
            debug!("starting reverse-shell-pipe");

            let result = crate::session::shell::start(id).await;
            endof_remote_shell_tx.send(()).unwrap();

            debug!("finished reverse-shell-pipe");

            return result;
        });

        let signal_register = rl.signal_register();

        // The purpoes of this task is to ignore all the Interruption(^C) between local shell mode & remote
        // shell mode. As it's complicated to run this in only interval of local shell mode &
        // remote shell mode, this is executed while remote shell pipe is running. And when remote
        // shell pipe is finished, this task will end.
        // In remote shell pipe, user input is polled. So while remote shell pipe is running,
        // signal handler won't receive any Interruptions(^C).
        let interrupt_ignoring = tokio::spawn(async move {
            debug!("{}", "started signal_canceler");
            let sig_reg_pair = signal_register;

            loop {
                let sig_reg_locked = sig_reg_pair.0.lock().unwrap();
                sig_reg_locked.catch_ctrl();
                let sig_reg_locked = sig_reg_pair.1.wait(sig_reg_locked).unwrap();

                // If remote shell pipe is finished, the sender sends a signal to tell it
                // to receiver here. Then, this has_changed() will cause an error because of
                // drop of the sender caused by select! macro. However, it is possible to detect the end
                // of remote shell pipe by detecting drop.
                match endof_remote_shell_rx.has_changed() {
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

        // This runs both remote_shell_pipe and interrupt_ignoring.
        // interrupt_ignoring will never stop naturally so, when reverse_shell_pipe is finished, it
        // will be droped. However, even if interrupt_ignoring is droped, its future cleanup isn't
        // done. So, it is necessary to tell the end of remote_shell_pipe to interrupt_ignoring. As
        // interrupt_ignoring is blocking to receive an Interruption(^C), by generating a Fake Interruption
        // the block will end and then, interrupt_ignoring will detect the end of remote_shell_pipe.
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

                // Fake Interruption
                let sig_reg_pair = rl.signal_register();
                sig_reg_pair.1.notify_all();
            }
            _ = interrupt_ignoring => {}
        }

        Ok(())
    }
}
