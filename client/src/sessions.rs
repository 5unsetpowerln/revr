use anyhow::Result;
use clap::Parser;

use crate::command::ArgsParser;

#[derive(Parser)]
struct Args {
    id: Option<usize>,
}

trait ToStringFields {
    fn to_string_fields(&self) -> Vec<String>;
}

macro_rules! to_string_fields_impl {
    ($struct_name:ident, $( $field:ident ),* ) => {
        impl ToStringFields for $struct_name {
            fn to_string_fields(&self) -> Vec<String> {
                vec![
                    $(
                        format!("{}: {}", stringify!($field), self.$field.to_string())
                    ),*
                ]
            }
        }
    }
}

pub fn sessions(args: &[&str]) -> Result<()> {
    let args = Args::parse_args("sessions", args)?;

    if args.id.is_none() {
        use prettytable::{row, Table};

        let metadatas = super::revshell::session::get_metadatas();
        let mut table = Table::new();

        table.add_row(row!["id", "address"]);
        for metadata in metadatas {
            table.add_row(row![
                metadata.id.to_string(),
                metadata.remote_addr.to_string()
            ]);
        }

        println!("{}", table);
        return Ok(());
    }
    
    let id = args.id.unwrap();


    Ok(())
}
