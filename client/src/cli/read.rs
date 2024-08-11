use std::{
    io::{self, Read},
    os::fd::AsRawFd,
};

use anyhow::Result;
use mio::{unix::SourceFd, Events, Interest, Poll, Token};
use termion::raw::IntoRawMode;

use crate::cli::ascii::char_to_ctrl;

fn read_until(until_list: &[u8]) -> Result<Vec<u8>> {
    let mut stdout = io::stdout().into_raw_mode().unwrap();
    let mut stdin = io::stdin().lock();
    let fd = io::stdin().as_raw_fd();

    let mut poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(1024);

    // register the fd of stdin
    poll.registry()
        .register(
            &mut SourceFd(&fd),
            Token(0),
            Interest::READABLE | Interest::WRITABLE,
        )
        .unwrap();

    let mut received_bytes = [0; 1];
    let mut local_buffer = Vec::new();

    let exit_ctrl_char = char_to_ctrl(b'D')?;
    let example_keybind_to_switch_buffering = char_to_ctrl(b'I')?;
    // let example_keybind_to_stop_buffering_input = char_to_ctrl(b'I')?;
    let mut is_buffering = false;

    loop {
        // waiting for event
        poll.poll(&mut events, None).unwrap();

        for event in &events {
            if event.token() == Token(0) && event.is_readable() {
                match stdin.read(&mut received_bytes) {
                    Ok(1) => {
                        let received_byte = received_bytes[0];
                        if received_byte == b'\r' {
                            return Ok(local_buffer);
                        }
                        local_buffer.push(received_byte);
                    }
                    Ok(_) => {
                        panic!("recieved EOF from stdin")
                    }
                    Err(e) => {
                        eprintln!("Error reading stdin: {}", e);
                        break;
                    }
                }
            }
        }
    }
}
