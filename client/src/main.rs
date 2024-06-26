mod cli;
mod command;
mod listen;
mod revshell;
mod sessions;

use cli::color;
use log::{error, Level};
use std::io::Write;

fn setup_logger() {
    std::env::set_var("RUST_LOG", "info");

    env_logger::builder()
        .format(|buf, record| match record.level() {
            Level::Error => writeln!(buf, " {} {}", color::red("+"), record.args()),
            Level::Debug => writeln!(buf, " {} {}", color::green("+"), record.args()),
            Level::Info => writeln!(buf, " {} {}", color::cyan("+"), record.args()),
            Level::Warn => writeln!(buf, " {} {}", color::yellow("+"), record.args()),
            Level::Trace => writeln!(buf, " {} {}", color::gray("+"), record.args()),
        })
        .init();
}

// #[tokio::main]
fn main() {
    let mut rl = rustyline::DefaultEditor::new().unwrap();
    let commands = command::get_commands();
    setup_logger();

    loop {
        let line = rl.readline(&format!("{} ", color::red("revr>"))).unwrap();

        let parts: Vec<&str> = line.split_whitespace().collect();
        let command_name = match parts.first() {
            Some(c) => c,
            None => continue,
        };
        let args = parts.get(1..).unwrap_or(&[]);

        if let Some(command) = commands.get(command_name) {
            if let Err(e) = (command.func)(args) {
                error!("{}", e)
            }
        } else {
            error!("unknown command: {}", command_name);
        }
    }
}
