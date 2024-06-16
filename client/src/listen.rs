use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    port: u16,
}

pub fn listen(args: &[&str]) -> Result<(), &'static str> {
    let args = if let Ok(a) = Args::try_parse_from(args) {
        a
    } else {
        return Err("failed to parse args");
    };
    Ok(())
}
