pub mod ascii;
pub mod color;
pub mod error;

use once_cell::sync::Lazy;

pub static PROMPT: Lazy<String> = Lazy::new(|| color::red("revr> "));
pub static CONFIRM_PROMPT: &str = "exit? (y/n)";

pub fn confirm(prompt: &str) -> bool {
    let mut rl = rustyline::DefaultEditor::new().unwrap();
    loop {
        match rl.readline(prompt) {
            Ok(line) => {
                if line == "y" {
                    return true;
                }
                return false;
            }
            _ => return false,
        }
    }
}
