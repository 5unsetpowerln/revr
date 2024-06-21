use std::{collections::HashMap, process};

use anyhow::{anyhow, bail, Error, Result};
use clap::Parser;

pub struct Command {
    pub description: String,
    pub func: fn(&[&str]) -> Result<(), Error>,
}

impl Command {
    pub fn new(description: &str, func: fn(&[&str]) -> Result<(), Error>) -> Self {
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

pub fn get_commands() -> HashMap<&'static str, Command> {
    let mut commands: HashMap<&str, Command> = HashMap::new();
    commands.insert("exit", Command::new("exit revr", |_| process::exit(0)));

    commands.insert(
        "listen",
        Command::new("start waiting for reverse shell", super::listen::listen),
    );

    commands.insert(
        "sessions",
        Command::new("manage reverse shell sessions", super::sessions::sessions),
    );

    commands.insert(
        "help",
        Command::new("show help", |args| {
            let commands = get_commands();
            let command_name = args.first();

            if command_name.is_none() {
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
            }

            let command_name = command_name.unwrap();
            let command = commands.get(command_name);
            if command.is_none() {
                bail!(anyhow!("unknown command: {}", command_name));
            }

            let command = command.unwrap();
            let command_args = if let Some(a) = args.get(1..) {
                a
            } else {
                &[""]
            };
            let _ = (command.func)(command_args);
            Ok(())
        }),
    );

    commands
}
