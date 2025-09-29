use std::{
    os::unix::{io::{AsRawFd, OwnedFd}, process::CommandExt},
    process::{Child, Command, Stdio},
};

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
        // Create cells with current background color
        let mut clear_cell = Cell::default();
        clear_cell.set_style(self.current_style);
        self.grid = vec![vec![clear_cell; self.width as usize]; self.height as usize];
    }

    /// Helper method to scroll the screen contents up by one line.
    /// This is called when the cursor moves past the bottom of the screen.
    fn scroll_up(&mut self) {
        if !self.grid.is_empty() {
            // Remove the top row
            self.grid.remove(0);
            // Add a new empty row at the bottom with current background
            let mut clear_cell = Cell::default();
            clear_cell.set_style(self.current_style);
            self.grid.push(vec![clear_cell; self.width as usize]);
        }
    }
    
    fn handle_sgr_and_update_style(&mut self, params: &Params) {
        for param in params.iter().flat_map(|p| p.iter()) {
            log::debug!("SGR: Processing param {}", param);
            match *param {
                // Reset all attributes
                0 => self.current_style = Style::default(), 
                1 => self.current_style = self.current_style.add_modifier(Modifier::BOLD),
                2 => self.current_style = self.current_style.add_modifier(Modifier::DIM),
                3 => self.current_style = self.current_style.add_modifier(Modifier::ITALIC),
                4 => self.current_style = self.current_style.add_modifier(Modifier::UNDERLINED),
                5 => self.current_style = self.current_style.add_modifier(Modifier::SLOW_BLINK),
                6 => self.current_style = self.current_style.add_modifier(Modifier::RAPID_BLINK),
                7 => self.current_style = self.current_style.add_modifier(Modifier::REVERSED),
                8 => self.current_style = self.current_style.add_modifier(Modifier::HIDDEN),
                
                // Crossed out text. 
                // Wikipedia mentions this is not supported on Terminal.app.
                9 => self.current_style = self.current_style.add_modifier(Modifier::CROSSED_OUT),
                
                // No clue what this does. 
                10 => log::error!("SGR (UNIMPLEMENTED): Select font {}", param),
                
                // Supposedly allows you to select a different font. 
                // Not sure how to implement this, but if someone finds a use for it,
                // patches are welcome. 
                11..=19 => log::debug!("SGR (UNIMPLEMENTED): Select font {}", param),
                
                // Let's you use the Fraktur (Gothic) font. 
                // Not sure how to implement this, but if someone finds a use for it,
                // patches are welcome. 
                20 => log::error!("SGR (UNIMPLEMENTED): Fraktur font"),
                
                // Let's you use the double-struck font. 
                21 => log::debug!("SGR (UNIMPLEMENTED): Double-struck font"),
                
                22 => self.current_style = self.current_style.remove_modifier(Modifier::BOLD),
                
                // Removes italic from the text. Wikipedia mentions it also removes "blackletter".
                // This probably means it also removes the Fraktur font, but 
                // since that's not implemented, we'll just remove the italic modifier.
                23 => self.current_style = self.current_style.remove_modifier(Modifier::ITALIC),
                24 => self.current_style = self.current_style.remove_modifier(Modifier::UNDERLINED),
                
                // Removes blink from the text. 
                25 => {
                    self.current_style = self.current_style.remove_modifier(Modifier::SLOW_BLINK);
                    self.current_style = self.current_style.remove_modifier(Modifier::RAPID_BLINK);
                },
                
                // Pretty sure this is unused.
                26 => log::error!("SGR (UNIMPLEMENTED): Remove double-struck"),
                27 => self.current_style = self.current_style.remove_modifier(Modifier::REVERSED),
                28 => self.current_style = self.current_style.remove_modifier(Modifier::HIDDEN),
                29 => self.current_style = self.current_style.remove_modifier(Modifier::HIDDEN),
                
                // Set foreground color. 
                30..=37 => self.current_style = self.current_style.fg(Color::Indexed(*param as u8 - 30)),
                
                // Set background color. 
                40..=47 => self.current_style = self.current_style.bg(Color::Indexed(*param as u8 - 40)),
                
                // Reset foreground color. 
                39 => self.current_style = self.current_style.fg(Color::Reset),
                
                // Reset background color. 
                49 => self.current_style = self.current_style.bg(Color::Reset),
                
                // There are a couple that are implemented in popular terminals, but
                // I'm going to ignore them for now. If there's a bug with kitty not 
                // properly displaying underline color, check Wikipedia and SGR codes
                // 58 and 59.
                _ => log::debug!("SGR: Unknown param {}", param),
            }
        }
        log::debug!("SGR: Final style set to {:?}", self.current_style);
    }
    
    /// Clears the line from the beginning to the cursor.
    fn erase_line_to_cursor(&mut self) {
        let mut clear_cell = Cell::default();
        clear_cell.set_style(self.current_style);

        for x in 0..=self.cursor.0 {
            if let Some(cell) = self.grid.get_mut(self.cursor.1 as usize).and_then(|row| row.get_mut(x as usize)) {
                *cell = clear_cell.clone();
            }
        }
    }

    /// Clears the line from the cursor to the end of the line.
    fn erase_line_from_cursor(&mut self) {
        let mut clear_cell = Cell::default();
        clear_cell.set_style(self.current_style);

        for x in self.cursor.0..self.width {
            if let Some(cell) = self.grid.get_mut(self.cursor.1 as usize).and_then(|row| row.get_mut(x as usize)) {
                *cell = clear_cell.clone();
            }
        }
    }
}

// This is the core of the ANSI parser. The `vte` crate calls these methods
// when it encounters specific ANSI escape codes in the byte stream.
impl Perform for Screen {
    /// Called when a printable character is encountered.
    fn print(&mut self, c: char) {
        log::debug!("Print char: '{}' at ({}, {})", c, self.cursor.0, self.cursor.1);
        
        // Handle automatic line wrapping if the cursor is at the end of the line.
        if self.cursor.0 >= self.width {
            self.cursor.0 = 0;
            self.cursor.1 += 1;
        }

        // If the cursor is past the last row, scroll the screen up.
        if self.cursor.1 >= self.height {
            self.scroll_up();
            self.cursor.1 = self.height - 1;
        }

        let (x, y) = self.cursor;

        // Place the character in the grid.
        if let Some(row) = self.grid.get_mut(y as usize) {
            if let Some(cell) = row.get_mut(x as usize) {
                cell.set_char(c);
                cell.set_style(self.current_style);
            }
        }

        // Advance the cursor.
        self.cursor.0 += 1;
    }

    /// Called for C0 control characters (like newline, backspace, etc.).
    fn execute(&mut self, byte: u8) {
        log::debug!("Execute control char: 0x{:02x}", byte);
        match byte {
            b'\n' => { // Line Feed (LF)
                // Move the cursor down one line AND to beginning of line
                // This is the default "newline mode" behavior
                if self.cursor.1 >= self.height - 1 {
                    self.scroll_up(); // Scroll up if the cursor is at the bottom of the screen
                } else {
                    self.cursor.1 += 1;
                }
                // Move to beginning of line (like \r\n)
                // This fixes the issue where the cursor would only move
                // down one line, and not reset to the beginning of the line.
                // See: https://stackoverflow.com/a/12747850
                self.cursor.0 = 0;

            }
            b'\r' => { // Carriage Return (CR)
                // Move the cursor to the beginning of the current line.
                self.cursor.0 = 0;
            }
            b'\x08' => { // Backspace
                // Move cursor left, but not past the beginning of the line.
                self.cursor.0 = self.cursor.0.saturating_sub(1);
            }
            _ => {
                log::debug!("Unhandled control char: 0x{:02x}", byte);
            } // Other C0 control codes are ignored for now.
        }
    }
    
    // VTE might call other methods - implement them as no-ops with logging
    // Not really sure what these are for, but they're required by the `Perform` trait.
    // The logs don't indicate that any of the animations use them, but they're required.
    fn hook(&mut self, params: &Params, _intermediates: &[u8], _ignore: bool, c: char) {
        log::debug!("Hook: {} params: {:?}", c, params);
    }
    
    fn put(&mut self, byte: u8) {
        log::debug!("Put: 0x{:02x}", byte);
    }
    
    fn unhook(&mut self) {
        log::debug!("Unhook");
    }
    
    fn osc_dispatch(&mut self, params: &[&[u8]], bell_terminated: bool) {
        log::debug!("OSC dispatch: params: {:?}, bell: {}", params, bell_terminated);
    }
    
    fn esc_dispatch(&mut self, intermediates: &[u8], _ignore: bool, byte: u8) {
        log::debug!("ESC dispatch: 0x{:02x} intermediates: {:?}", byte, intermediates);
    }

    /// Called for Control Sequence Introducer (CSI) commands.
    /// This is where we handle the actual animation commands that the animation may output. 
    /// See Part 2, Chapter 5 of https://vt100.net/docs/vt510-rm/contents.html for more details.
    fn csi_dispatch(&mut self, params: &Params, _intermediates: &[u8], _ignore: bool, c: char) {
        log::debug!("CSI command: '{}' with params: {:?}", c, params);
        match c {
            'A' => { // Cursor Up
                let count = params.iter().next().and_then(|p| p.first()).cloned().unwrap_or(1);
                log::debug!("Cursor Up by {}", count);
                self.cursor.1 = self.cursor.1.saturating_sub(count as u16);
            }
            'B' => { // Cursor Down
                let count = params.iter().next().and_then(|p| p.first()).cloned().unwrap_or(1);
                log::debug!("Cursor Down by {}", count);
                self.cursor.1 = (self.cursor.1 + count as u16).min(self.height - 1);
            }
            'C' => { // Cursor Forward (Right)
                let count = params.iter().next().and_then(|p| p.first()).cloned().unwrap_or(1);
                log::debug!("Cursor Right by {}", count);
                self.cursor.0 = (self.cursor.0 + count as u16).min(self.width - 1);
            }
            'D' => { // Cursor Backward (Left)
                let count = params.iter().next().and_then(|p| p.first()).cloned().unwrap_or(1);
                log::debug!("Cursor Left by {}", count);
                self.cursor.0 = self.cursor.0.saturating_sub(count as u16);
            }
            'd' => { // Line Position Absolute (VPA)
                // I couldn't find doucmentation about this command on wikipedia, but you can find it here:
                // https://vt100.net/docs/vt510-rm/VPA.html
                // https://ghostty.org/docs/vt/csi/vpa 

                // Essentially, this moves the cursor to the absolute row number.
                // The default is 1, so we subtract 1 to get the 0-based index.
                // The max is the height of the screen, so we clamp it.
                let row = params.iter().next().and_then(|p| p.first()).cloned().unwrap_or(1);
                log::debug!("Move cursor to row {}", row);
                self.cursor.1 = (row as u16).saturating_sub(1).min(self.height - 1);
            }
            'H' | 'f' => { // Cursor Position
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
            'J' => { // Erase in Display (ED)
                // Docs: https://vt100.net/docs/vt510-rm/ED.html
                // This is the command that clears the screen. 
                // Default to 0 if no parameter is provided. 
                let param = params.iter().next().and_then(|p| p.first()).cloned().unwrap_or(0);
                
                // We will clone this cell to clear the screen. 
                let mut clear_cell = Cell::default();
                clear_cell.set_style(self.current_style);
                
                match param {
                    0 => { // Erase from cursor to end of screen
                        self.erase_line_from_cursor();

                        // Clear all lines below the cursor
                        for y in (self.cursor.1 + 1)..self.height {
                            if let Some(row) = self.grid.get_mut(y as usize) {
                                for cell in row.iter_mut() {
                                    *cell = clear_cell.clone();
                                }
                            }
                        }
                    }
                    1 => { // Erase from cursor to beginning of screen
                        // Clear all lines above the cursor
                        for y in 0..self.cursor.1 {
                            if let Some(row) = self.grid.get_mut(y as usize) {
                                for cell in row.iter_mut() {
                                    *cell = clear_cell.clone();
                                }
                            }
                        }
                        self.erase_line_to_cursor();
                    }


                    // Erase entire screen (ED2) or delete all lines saved in the scrollback buffer (ED3).
                    // We don't have a scrollback buffer, so this is the same as 2.
                    // This isn't described in vt100.net, but it's listed on Wikipedia. 
                    // Note: This command does not affect the cursor position. 
                    2..=3 => self.clear(),
                    _ => {
                        log::debug!("ED: Unknown param {}", param);
                    }
                }
            }
            'm' => { // Select Graphic Rendition (SGR)
                // This CSI command is used to set the text style. 
                // Docs: https://vt100.net/docs/vt510-rm/SGR.html
                // Wikipedia: https://en.wikipedia.org/wiki/ANSI_escape_code#SGR  
                log::debug!("SGR params: {:?}", params);
                // An empty parameter list is equivalent to a parameter of 0 (reset).
                if params.is_empty() || (params.len() == 1 && params.iter().next().unwrap().is_empty()) {
                    log::debug!("SGR: Resetting style to default");
                    self.current_style = Style::default();
                    return;
                }

                // Process all parameters to build the new style
                self.handle_sgr_and_update_style(params);
            }
            'K' => { // Erase in Line (EL)
                let param = params.iter().next().and_then(|p| p.first()).cloned().unwrap_or(0);
                log::debug!("EL (Erase Line) param: {}", param);
                
                // We will clone this cell to clear the line. 
                let mut clear_cell = Cell::default();
                clear_cell.set_style(self.current_style);
                
                match param {
                    0 => self.erase_line_from_cursor(),
                    1 => { // Erase from beginning of line to cursor
                        self.erase_line_to_cursor();
                    }
                    2 => { // Erase entire line
                        if let Some(row) = self.grid.get_mut(self.cursor.1 as usize) {
                            for cell in row.iter_mut() {
                                *cell = clear_cell.clone();
                            }
                        }
                    }
                    _ => {}
                }
            }
            'l' => { // Reset Mode (RM)
                // This is used to reset the mode of the terminal. 
                // Docs: https://vt100.net/docs/vt510-rm/RM.html
                // The RM codes are listed in Table 5-8: https://vt100.net/docs/vt510-rm/DECRQM.html#T5-8
                //
                // I have no idea how to implement these, but so far I don't get any major errors,
                // so I'm going to leave them alone. AFAIK they won't cause any major issues, but 
                // some animations may rely on them, like astroterm. 
                for param in params.iter().flat_map(|p| p.iter()) {
                    log::debug!("Reset Mode: {}", param);
                    match *param {
                        5 => self.clear(),
                        _ => {
                            log::debug!("Unknown reset mode: {}", param);
                        }
                    }
                }
            }
            't' => { // Window manipulation
                log::debug!("Window manipulation command (ignored)");
                // These are terminal window operations - safe to ignore
            }
            _ => {
                log::debug!("Unhandled CSI command: '{}' with params: {:?}", c, params);
            }
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
        log::info!("Animation::new called with command: {}, args: {:?}, size: {}x{}", command, args, size.width, size.height);
        
        // 1. Create a new PTY
        let pty = openpty(None, None).ok()?;
        log::debug!("PTY created successfully");

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
        log::debug!("Child process spawned successfully");

        // Set the master PTY to non-blocking mode.
        fcntl(&pty.master, FcntlArg::F_SETFL(OFlag::O_NONBLOCK)).ok()?;
        log::debug!("PTY set to non-blocking mode");

        // 3. Initialize the VTE parser and screen model
        let screen = Screen::new(size.width, size.height);
        let parser = Parser::new();

        log::info!("Animation created successfully");
        Some(Self {
            child_process: child,
            pty_master: pty.master, // Directly move the master
            parser,
            screen,
        })
    }

    // On each tick, read from the PTY and feed the bytes to the parser.
    // Returns true if the screen was updated, false if no new data was available.
    pub fn update(&mut self) -> bool {
        let mut buffer = [0u8; 4096];
        let mut updated = false;
        let mut total_bytes = 0;
        
        // Keep reading until no more data is available
        loop {
            match nix::unistd::read(&self.pty_master, &mut buffer) {
                Ok(bytes_read) => {
                    if bytes_read > 0 {
                        total_bytes += bytes_read;
                        log::debug!("Read {} bytes: {:?}", bytes_read, std::str::from_utf8(&buffer[..bytes_read]).unwrap_or("[invalid utf8]"));
                        self.parser.advance(&mut self.screen, &buffer[..bytes_read]);
                        updated = true;
                    } else {
                        // bytes_read == 0 means EOF
                        log::debug!("PTY EOF");
                        break;
                    }
                }
                Err(nix::Error::EAGAIN) => {
                    // No more data available - break out of loop
                    if total_bytes == 0 {
                        log::debug!("No PTY data available");
                    }
                    break;
                }
                Err(e) => {
                    // A real error occurred - break out of loop
                    log::error!("PTY read error: {}", e);
                    break;
                }
            }
        }
        
        if total_bytes > 0 {
            log::debug!("Total bytes read this update: {}", total_bytes);
        }
        updated
    }
}

// Implement the `Widget` trait to draw the captured screen state.
impl Widget for &Animation {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Simply copy the cells from our internal screen model to the ratatui buffer.
        let mut non_empty_cells = 0;
        for y in 0..area.height.min(self.screen.height) {
            for x in 0..area.width.min(self.screen.width) {
                if let Some(cell) = self
                    .screen
                    .grid
                    .get(y as usize)
                    .and_then(|row| row.get(x as usize))
                {
                    // Count non-empty cells for debug
                    if cell.symbol() != " " && !cell.symbol().is_empty() {
                        non_empty_cells += 1;
                    }
                    buf[(area.x + x, area.y + y)] = cell.clone();
                }
            }
        }
        log::debug!("Rendered {} non-empty cells from screen {}x{}", non_empty_cells, self.screen.width, self.screen.height);
    }
}
