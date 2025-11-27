use anstyle::{AnsiColor, Style};
use std::io::{self, Write};

const YELLOW: Style = Style::new().fg_color(Some(anstyle::Color::Ansi(AnsiColor::Yellow)));
const BOLD: Style = Style::new().bold();

pub fn warn(msg: &str) {
    let mut stderr = io::stderr();
    let _ = writeln!(stderr, "{YELLOW}{BOLD}warning:{BOLD:#}{YELLOW:#} {msg}");
}
