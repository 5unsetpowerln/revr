// mod back;
mod cli;
mod cmd;
// mod config;
// mod download;
mod errors;
// mod listen;
mod session;
// mod sessions;
mod shell;
// mod upload;

// use chan::chan_select;
use clap::Parser;
use cli::{color, confirm, CONFIRM_PROMPT, PROMPT};
use log::{error, Level};
use rustyline::error::ReadlineError;
use shell::run;
use std::{
    io::Write,
    sync::mpsc::{self, Receiver},
};
use tokio::{select, sync::broadcast};

fn setup_logger() {
    std::env::set_var("RUST_LOG", "info");

    env_logger::builder()
        .format(|buf, record| match record.level() {
            Level::Error => writeln!(buf, "{} {}", color::red("+"), record.args()),
            Level::Debug => writeln!(buf, "{} {}", color::green("+"), record.args()),
            Level::Info => writeln!(buf, "{} {}", color::cyan("+"), record.args()),
            Level::Warn => writeln!(buf, "{} {}", color::yellow("+"), record.args()),
            Level::Trace => writeln!(buf, "{} {}", color::gray("+"), record.args()),
        })
        .init();
}

#[derive(Parser, Debug)]
struct Args {
    port: Option<u16>,
}

pub struct LocalState {
    session_ctx: Option<usize>,
    is_exited: bool,
}

impl LocalState {
    pub fn new() -> Self {
        Self {
            session_ctx: None,
            is_exited: false,
        }
    }
}

// fn main() {
//     let mut args = Args::parse();
//     let mut rl = rustyline::DefaultEditor::new().unwrap();
//     let commands = shell::get_commands();
//     let mut local_state = LocalState::new();
//     setup_logger();
//     let (sigint_sender, _sigint_receiver) = broadcast::channel(10);
//     let sigint_sender_clone = sigint_sender.clone();
//     ctrlc::set_handler(move || {
//         sigint_sender_clone.send(()).unwrap();
//     })
//     .unwrap();

//     'local_loop: loop {
//         if local_state.is_exited {
//             break;
//         }
//         let line = if let Some(port) = args.port {
//             args.port = None;
//             format!("listen {}", port)
//         } else {
//             match rl.readline(&color::red(&PROMPT)) {
//                 Ok(line) => line,
//                 Err(ReadlineError::Eof) => {
//                     if confirm(&CONFIRM_PROMPT) {
//                         break;
//                     } else {
//                         continue;
//                     }
//                 }
//                 _ => continue 'local_loop,
//             }
//         };

//         let parts: Vec<&str> = line.split_whitespace().collect();
//         let command_name = match parts.first() {
//             Some(c) => c,
//             None => continue,
//         };
//         let args = parts.get(1..).unwrap_or(&[]);

//         if let Some(command) = commands.get(command_name) {
//             if let Err(e) = (command.func)(args, &mut local_state, &mut sigint_sender.subscribe()) {
//                 error!("{}", e)
//             }
//         } else {
//             error!("unknown command: {}", command_name);
//         }
//     }
// }

#[tokio::main]
async fn main() {
    setup_logger();

    if let Err(err) = run().await {
        error!("{}", err);
        for cause in err.chain() {
            eprintln!("because: {}", cause);
        }
    }
}
