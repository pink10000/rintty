use clap::Parser;
use nix::unistd::ForkResult;
use nix::{fcntl, sys::stat, unistd};
use simplelog::*;
use std::{fs::File, io, os::unix::io::AsRawFd};

mod app;
mod auth;
mod tui;
mod utils;
mod animation;

/// A TUI login screen for rintty, a modern replacement for agetty.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The path to the TTY device. If omitted, rintty runs in test mode.
    tty_path: Option<String>,
    
    /// Show password in plain text instead of masking it
    #[arg(short = 'p', long)]
    show_password: bool,

    /// The command to run for the background animation.
    #[arg(long)]
    animation: Option<String>,

    /// Logging
    #[arg(short = 'l', long)]
    logging: bool,

    /// Increase loggingverbosity (can be used multiple times: -v, -vv, -vvv, -vvvv)
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    verbose: u8,
}

fn main() -> io::Result<()> {
    let cli: Cli = Cli::parse();
    
    if cli.logging {
        // Set up logging to file
        WriteLogger::init(
            LevelFilter::Debug,
            Config::default(),
            File::create("rintty.log").unwrap(),
        ).unwrap();
    }

    log::info!("Rintty starting up");
    // TODO: need to make sure animation exists and is executable

    if let Some(ref path) = cli.tty_path {
        // Forking allows setsid() to succeed. Otherwise, setsid() will fail with EPERM as it is a process group leader.
        // https://man7.org/linux/man-pages/man2/setsid.2.html
        match unsafe { unistd::fork() } {
            Ok(ForkResult::Parent { .. }) => std::process::exit(0),
            Ok(ForkResult::Child) => {
                // Child process continues. It is no longer a process group leader.

                unistd::setsid().unwrap_or_else(|e| panic!("Child: setsid failed: {}", e));

                let tty_fd = fcntl::open(path.as_str(), fcntl::OFlag::O_RDWR, stat::Mode::empty())
                    .unwrap_or_else(|e| panic!("fcntl::open of {} failed: {}", path, e));

                unsafe {
                    let result = libc::ioctl(tty_fd.as_raw_fd(), libc::TIOCSCTTY, 1);
                    if result == -1 {
                        // Get the last OS error to see why ioctl failed.
                        let err = io::Error::last_os_error();
                        panic!("ioctl(TIOCSCTTY) failed: {}", err);
                    }
                }

                // Redirect stdin, stdout, and stderr to the TTY file descriptor.
                // From this point on, all `println!`, `stdout()`, etc. will go to the TTY.
                unistd::dup2_stdin(&tty_fd)?;
                unistd::dup2_stdout(&tty_fd)?;
                unistd::dup2_stderr(&tty_fd)?;
            }
            Err(e) => {
                panic!("fork failed: {}", e);
            }
        }
    } 
    tui::run(cli)?;

    Ok(())
}