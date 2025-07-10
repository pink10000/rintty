use std::io::{self, stdout};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
use crossterm::{execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen}};
use ratatui::{
    prelude::{Constraint, CrosstermBackend, Direction, Layout, Rect, Style, Terminal},
    widgets::{Block, Borders, Padding, Paragraph},
};

use crate::app::{App, ActiveField};
use crate::utils;


pub fn run() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout: io::Stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let mut app = App::new();

    loop {
        terminal.draw(|frame| {
            let frame_area: Rect = frame.area();
            let login_form_rect: Rect = login_form_rect(15, frame_area);

            let login_block = Block::default()
                .title("Login")
                .borders(Borders::ALL)
                .padding(Padding::horizontal(1));

            let form_layout = Layout::default()
                .direction(Direction::Vertical)
                // .margin(1)
                .constraints([Constraint::Length(3), Constraint::Length(3)])
                .split(login_block.inner(login_form_rect));

            frame.render_widget(login_block, login_form_rect);

            // We subtract 2 from the width to account for the borders.
            let username_input = Paragraph::new(utils::last_n_chars(
                app.username.as_str(),
                (form_layout[0].width - 2) as usize,
            ))
            .block(Block::default().borders(Borders::ALL).title("Username"))
            .style(match app.active_field {
                ActiveField::Username => Style::default().fg(ratatui::style::Color::LightMagenta),
                _ => Style::default(),
            });
            frame.render_widget(username_input, form_layout[0]);

            let password_masked = "*".repeat(
                utils::last_n_chars(app.password.as_str(), (form_layout[1].width - 2) as usize)
                    .len(),
            );
            let password_input = Paragraph::new(password_masked)
                .block(Block::default().borders(Borders::ALL).title("Password"))
                .style(match app.active_field {
                    ActiveField::Password => {
                        Style::default().fg(ratatui::style::Color::LightMagenta)
                    }
                    _ => Style::default(),
                });
            frame.render_widget(password_input, form_layout[1]);

            match app.active_field {
                ActiveField::Username => {
                    if app.username.is_empty() {
                        frame.set_cursor_position((form_layout[0].x + 1, form_layout[0].y + 1));
                    } else if form_layout[0].width > app.username.len() as u16 + 1 {
                        frame.set_cursor_position((
                            form_layout[0].x + app.username.len() as u16 + 1,
                            form_layout[0].y + 1,
                        ));
                    }
                }
                ActiveField::Password => {
                    if app.password.is_empty() {
                        frame.set_cursor_position((form_layout[1].x + 1, form_layout[1].y + 1));
                    } else if form_layout[1].width > app.password.len() as u16 + 1 {
                        frame.set_cursor_position((
                            form_layout[1].x + app.password.len() as u16 + 1,
                            form_layout[1].y + 1,
                        ));
                    }
                }
            }
        })?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Esc => break,
                        KeyCode::Tab => {
                            app.active_field = match app.active_field {
                                ActiveField::Username => ActiveField::Password,
                                ActiveField::Password => ActiveField::Username,
                            };
                        }
                        KeyCode::Char(c) => {
                            match app.active_field {
                                ActiveField::Username => app.username.push(c),
                                ActiveField::Password => app.password.push(c),
                            };
                        }
                        KeyCode::Backspace => {
                            match app.active_field {
                                ActiveField::Username => {
                                    app.username.pop();
                                }
                                ActiveField::Password => {
                                    app.password.pop();
                                }
                            };
                        }
                        KeyCode::Enter => {
                            if app.username.is_empty() || app.password.is_empty() {
                                continue;
                            }
                            let service = "login";
                            let mut auth = pam::Authenticator::with_password(service).unwrap();
                            auth.get_handler()
                                .set_credentials(app.username.as_str(), app.password.as_str());

                            if auth.authenticate().is_ok() && auth.open_session().is_ok() {
                                println!("Authentication successful!");
                                break;
                            } else {
                                app.username.clear();
                                app.password.clear();
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // TEARDOWN
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

/// Helper function to create the centered rectangle for the login form.
fn login_form_rect(percent_x: u16, r: Rect) -> Rect {
    let popup_width_f = r.width as f32 * (percent_x as f32 / 100.0);

    let final_width = (popup_width_f.max(30.0) as u16).min(r.width);
    let final_height = 8;

    let horizontal_margin = r.width.saturating_sub(final_width) / 2;
    let vertical_margin = r.height.saturating_sub(final_height) / 2;

    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(vertical_margin),
            Constraint::Length(final_height),
            Constraint::Length(vertical_margin),
        ])
        .split(r);

    // Create the layout for horizontal centering on the middle vertical chunk.
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(horizontal_margin),
            Constraint::Length(final_width),
            Constraint::Length(horizontal_margin),
        ])
        .split(popup_layout[1])[1]
}
