#[allow(dead_code)]
pub fn red(text: &str) -> String {
    format!("\x1b[31m{}\x1b[0m", text)
}
#[allow(dead_code)]
pub fn green(text: &str) -> String {
    format!("\x1b[32m{}\x1b[0m", text)
}
#[allow(dead_code)]
pub fn yellow(text: &str) -> String {
    format!("\x1b[33m{}\x1b[0m", text)
}
#[allow(dead_code)]
pub fn blue(text: &str) -> String {
    format!("\x1b[34m{}\x1b[0m", text)
}
#[allow(dead_code)]
pub fn magenta(text: &str) -> String {
    format!("\x1b[35m{}\x1b[0m", text)
}
#[allow(dead_code)]
pub fn cyan(text: &str) -> String {
    format!("\x1b[36m{}\x1b[0m", text)
}
#[allow(dead_code)]
pub fn gray(text: &str) -> String {
    format!("\x1b[37m{}\x1b[0m", text)
}
#[allow(dead_code)]
pub fn black(text: &str) -> String {
    format!("\x1b[30m{}\x1b[0m", text)
}
