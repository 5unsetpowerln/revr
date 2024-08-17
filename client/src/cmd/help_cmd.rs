use crate::errors::*;

use crate::shell::Shell;

#[inline]
fn help(name: &str, descr: &str) {
    println!("    \x1b[32m{:13}\x1b[0m {}", name, descr);
}

pub fn run(_rl: &mut Shell, _args: &[String]) -> Result<()> {
    println!("\n\x1b[33mCOMMANDS:\x1b[0m");
    help("remote", "manages reverse shell sessions");
    help("listen", "starts waiting for reverse shell");
    help("help", "prints this message");
    println!("\nRun <command> -h for more help.\n");

    Ok(())
}
