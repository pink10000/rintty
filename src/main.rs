use clap::Parser;
use std::io;

mod app;
mod tui;
mod utils;

/// A TUI login screen for rintty, a modern replacement for agetty.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The path to the TTY device. If omitted, rintty runs in test mode.
    tty_path: Option<String>,
}

fn main() -> io::Result<()> {
    let cli: Cli = Cli::parse();
    if let Some(path) = cli.tty_path {
        println!("Normal Mode: Would take over TTY at {}", path);
    } else {
        tui::run()?;
    }

    Ok(())
}