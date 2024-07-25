use std::{collections::HashMap, future::Future};

use anyhow::{anyhow, bail, Error, Result};
use clap::Parser;
use tokio::{runtime::Runtime, select, sync::broadcast::Receiver};

use crate::{
    cli::{confirm, CONFIRM_PROMPT},
    LocalState,
};

pub struct Command {
    pub description: String,
    pub func:
        fn(&[&str], &mut LocalState, interrupt_receiver: &mut Receiver<()>) -> Result<(), Error>,
}

impl Command {
    pub fn new(
        description: &str,
        func: fn(
            &[&str],
            &mut LocalState,
            interrupt_receiver: &mut Receiver<()>,
        ) -> Result<(), Error>,
    ) -> Self {
        Self {
            description: description.to_string(),
            func,
        }
    }
}

pub trait ArgsParser<T>: Parser {
    fn parse_args(command: &str, args: &[&str]) -> Result<T>;
}

impl<T> ArgsParser<T> for T
where
    T: Parser,
{
    fn parse_args(command: &str, args: &[&str]) -> Result<T> {
        let mut args_with_command = vec![command];
        args_with_command.extend_from_slice(args);
        let args = T::try_parse_from(args_with_command.iter())?;
        Ok(args)
    }
}

enum RunningResult<T> {
    Ok(T),
    Interrupted,
}

fn run<F: Future>(future: F, sigint_receiver: &mut Receiver<()>) -> Result<()>
where
    F: Future<Output = Result<()>>,
{
    async fn run_inner<F: Future>(future: F, sigint_receiver: &mut Receiver<()>) -> Result<()>
    where
        F: Future<Output = Result<()>>,
    {
        // async fn recv(r: Receiver<()>) {
        // r.recv().unwrap();
        // }
        select! {
                _ = sigint_receiver.recv() => {
                    bail!("Interrupted")
                }
                result = future => {
                    return result
                }
        }
    }

    let runtime = Runtime::new().unwrap();
    runtime.block_on(run_inner(future, sigint_receiver))
}

pub fn get_commands() -> HashMap<&'static str, Command> {
    // create_run_function!();
    let mut commands: HashMap<&str, Command> = HashMap::new();
    commands.insert(
        "exit",
        Command::new("exit revr", |_, local_state, _| {
            if confirm(CONFIRM_PROMPT) {
                local_state.is_exited = true;
                Ok(())
            } else {
                Ok(())
            }
        }),
    );

    commands.insert(
        "listen",
        Command::new(
            "start waiting for reverse shell",
            |args, local_state, sigint_receiver| {
                run(super::listen::listen(args, local_state), sigint_receiver)
            },
        ),
    );

    commands.insert(
        "sessions",
        Command::new(
            "manage reverse shell sessions",
            |args, app_state, sigint_receiver| {
                run(super::sessions::sessions(args, app_state), sigint_receiver)
            },
        ),
    );

    commands.insert(
        "back",
        Command::new(
            "alias for sessions <id>",
            |args, local_state, sigint_receiver| {
                run(super::back::back(args, local_state), sigint_receiver)
            },
        ),
    );

    commands.insert(
        "upload",
        Command::new(
            "upload a file to remote server",
            |args, local_state, sigint_receiver| {
                run(super::upload::upload(args, local_state), sigint_receiver)
            },
        ),
    );

    commands.insert(
        "help",
        Command::new("show help", |_, local_state, _| {
            let commands = get_commands();

            let name_max_length = commands.keys().map(|&name| name.len()).max().unwrap_or(0);

            for (name, command) in commands {
                println!(
                    "{:<width$}\t{}",
                    name,
                    command.description,
                    width = name_max_length
                );
            }
            return Ok(());
        }),
    );

    commands
}
