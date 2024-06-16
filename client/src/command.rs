use std::{collections::HashMap, process};

pub struct Command {
    pub description: String,
    pub func: fn(&[&str]) -> Result<(), &'static str>,
}

impl Command {
    pub fn new(description: &str, func: fn(&[&str]) -> Result<(), &'static str>) -> Self {
        Self {
            description: description.to_string(),
            func,
        }
    }
}

pub fn get_commands() -> HashMap<&'static str, Command> {
    let mut commands: HashMap<&str, Command> = HashMap::new();
    commands.insert("exit", Command::new("exit revr", |_args| process::exit(0)));

    commands.insert(
        "listen",
        Command::new("start waiting for reverse shell", crate::listen::listen),
    );

    commands.insert(
        "help",
        Command::new("show help", |_args| {
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
            Ok(())
        }),
    );

    commands
}
