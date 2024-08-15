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
