use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Stdio proxy is in use")]
    ProxyInUse,
    #[error("Stdio proxy is still waiting for a byte")]
    ProxyIsWaiting,
    // #[error("interrupted")]
    // Interrupted,
}
