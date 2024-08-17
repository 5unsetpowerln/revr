use crate::shell::Shell;
use anyhow::Result;

pub trait Cmd: clap::Parser + Sized {
    async fn run(self, rl: &mut Shell) -> Result<()>;

    #[inline]
    async fn run_str(rl: &mut Shell, args: &[String]) -> Result<()> {
        let args = Self::try_parse_from(args)?;
        args.run(rl).await
    }
}

pub mod back_cmd;
pub mod download_cmd;
pub mod help_cmd;
pub mod listen_cmd;
pub mod remote_cmd;
pub mod upload_cmd;
