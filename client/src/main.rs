mod command;
mod listen;
mod session;

fn main() {
    let mut rl = rustyline::DefaultEditor::new().unwrap();

    let commands = command::get_commands();

    loop {
        let line = rl.readline("revr> ").unwrap();

        let parts: Vec<&str> = line.split_whitespace().collect();
        let command_name = match parts.first() {
            Some(c) => c,
            None => continue,
        };
        let args = parts.get(1..).unwrap_or(&[""]);

        if let Some(command) = commands.get(command_name) {
            let _ = (command.func)(args);
        } else {
            println!("unknown command: {}", command_name);
        }
    }
}
