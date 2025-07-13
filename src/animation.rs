use std::os::unix::io::{AsRawFd, FromRawFd, OwnedFd};
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};

use nix::{
    fcntl::{fcntl, FcntlArg, OFlag},
    pty::{openpty, Winsize},
};
use ratatui::{
    buffer::{Buffer, Cell},
    layout::Rect,
    prelude::*,
    widgets::Widget,
};
use vte::{Params, Parser, Perform};

// Represents the state of the child terminal's screen.
#[derive(Debug, Clone)]
struct Screen {
    grid: Vec<Vec<Cell>>,
    cursor: (u16, u16),
    current_style: Style,
    width: u16,
    height: u16,
}

impl Screen {
    fn new(width: u16, height: u16) -> Self {
        Self {
            grid: vec![vec![Cell::default(); width as usize]; height as usize],
            cursor: (0, 0),
            current_style: Style::default(),
            width,
            height,
        }
    }

    fn clear(&mut self) {
        self.grid = vec![vec![Cell::default(); self.width as usize]; self.height as usize];
    }
}

// This is the core of the ANSI parser. The `vte` crate calls these methods
// when it encounters specific ANSI escape codes in the byte stream.
impl Perform for Screen {
    fn print(&mut self, c: char) {
        let (x, y) = self.cursor;
        if x < self.width && y < self.height {
            if let Some(row) = self.grid.get_mut(y as usize) {
                if let Some(cell) = row.get_mut(x as usize) {
                    cell.set_char(c);
                    cell.set_style(self.current_style);
                }
            }
        }
        self.cursor.0 = (self.cursor.0 + 1).min(self.width - 1);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                self.cursor.1 = (self.cursor.1 + 1).min(self.height - 1);
                self.cursor.0 = 0;
            }
            b'\r' => {
                self.cursor.0 = 0;
            }
            _ => {}
        }
    }

    fn csi_dispatch(&mut self, params: &Params, _intermediates: &[u8], _ignore: bool, c: char) {
        match c {
            'H' | 'f' => {
                let row = params
                    .iter()
                    .next()
                    .and_then(|p| p.first())
                    .map(|&v| v as u16)
                    .unwrap_or(1)
                    .saturating_sub(1);
                let col = params
                    .iter()
                    .nth(1)
                    .and_then(|p| p.first())
                    .map(|&v| v as u16)
                    .unwrap_or(1)
                    .saturating_sub(1);
                self.cursor = (col.min(self.width - 1), row.min(self.height - 1));
            }
            'J' => {
                if let Some(param) = params.iter().next().and_then(|p| p.first()) {
                    if *param == 2 {
                        self.clear();
                        self.cursor = (0, 0);
                    }
                }
            }
            'm' => {
                let mut fg = None;
                let mut bg = None;
                for param in params.iter().flat_map(|p| p.iter()) {
                    match *param {
                        0 => self.current_style = Style::default(),
                        30..=37 => fg = Some(Color::Indexed(*param as u8 - 30)),
                        40..=47 => bg = Some(Color::Indexed(*param as u8 - 40)),
                        _ => {}
                    }
                }
                if let Some(color) = fg {
                    self.current_style = self.current_style.fg(color);
                }
                if let Some(color) = bg {
                    self.current_style = self.current_style.bg(color);
                }
            }
            _ => {}
        }
    }
}

// The main animation struct that holds the child process and its screen state.
pub struct Animation {
    #[allow(dead_code)]
    child_process: Child,
    pty_master: OwnedFd,
    parser: Parser,
    screen: Screen,
}

// When the Animation struct is dropped, we must ensure the child process is terminated,
// preventing orphan processes from consuming CPU in the background.
impl Drop for Animation {
    fn drop(&mut self) {
        let _ = self.child_process.kill();
        let _ = self.child_process.wait();
    }
}

impl Animation {
    pub fn new(command: &str, args: &[&str], size: Rect) -> Option<Self> {
        // 1. Create a new PTY
        let pty = openpty(None, None).ok()?;

        // Set the window size of the PTY slave so the child process knows its dimensions.
        let winsize = Winsize {
            ws_row: size.height,
            ws_col: size.width,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        // Set the window size of the PTY slave using ioctl.
        unsafe {
            libc::ioctl(pty.slave.as_raw_fd(), libc::TIOCSWINSZ, &winsize);
        }
        let slave = pty.slave;

        // 2. Spawn the child process
        let mut cmd = Command::new(command);
        cmd.args(args);

        // Clone the slave FD for the pre_exec closure.
        // This moves the clone into the closure, leaving the original `slave` available.
        let slave_for_closure = slave.try_clone().ok()?;

        // This is the safe way to handle the PTY slave.
        // We pass ownership of the slave file descriptor to the child process.
        // The `pre_exec` closure runs in the child process right before `exec` is called.
        unsafe {
            cmd.pre_exec(move || {
                nix::unistd::setsid()?;
                libc::ioctl(slave_for_closure.as_raw_fd(), libc::TIOCSCTTY, 1);
                Ok(())
            });
            cmd.stdin(Stdio::from(slave.try_clone().ok()?));
            cmd.stdout(Stdio::from(slave.try_clone().ok()?));
            cmd.stderr(Stdio::from(slave));
        }

        let child = cmd.spawn().ok()?;

        // Set the master PTY to non-blocking mode.
        fcntl(&pty.master, FcntlArg::F_SETFL(OFlag::O_NONBLOCK)).ok()?;

        // 3. Initialize the VTE parser and screen model
        let screen = Screen::new(size.width, size.height);
        let parser = Parser::new();

        Some(Self {
            child_process: child,
            pty_master: pty.master, // Directly move the master
            parser,
            screen,
        })
    }

    // On each tick, read from the PTY and feed the bytes to the parser.
    pub fn update(&mut self) {
        let mut buffer = [0u8; 4096];
        match nix::unistd::read(&self.pty_master, &mut buffer) {
            Ok(bytes_read) => {
                self.parser.advance(&mut self.screen, &buffer[..bytes_read]);
            }
            Err(nix::Error::EAGAIN) => {
                // Expected in non-blocking mode when no data is available.
            }
            Err(_) => {
                // A real error occurred.
            }
        }
    }
}

// Implement the `Widget` trait to draw the captured screen state.
impl Widget for &Animation {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Simply copy the cells from our internal screen model to the ratatui buffer.
        for y in 0..area.height.min(self.screen.height) {
            for x in 0..area.width.min(self.screen.width) {
                if let Some(cell) = self
                    .screen
                    .grid
                    .get(y as usize)
                    .and_then(|row| row.get(x as usize))
                {
                    *buf.get_mut(area.x + x, area.y + y) = cell.clone();
                }
            }
        }
    }
}
