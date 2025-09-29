/// The purpose of this file is to test the animation module without dealing
/// with the login form. It focuses solely on the Animation creation, VTE parsing
/// PTY communication, and the Screen rendering. 

use std::time::{Duration, Instant};
use std::io::{self, stdout};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    event::{self, Event, KeyCode},
    cursor,
};
use ratatui::{
    prelude::*,
    widgets::*,
    style::Color,
};
use simplelog::*;
use std::fs::File;

// Import our animation module
use rintty::animation::Animation;

fn main() -> io::Result<()> {
    // Set up logging
    WriteLogger::init(
        LevelFilter::Debug,
        Config::default(),
        File::create("test_animation.log").unwrap(),
    ).unwrap();

    log::info!("Test animation starting");

    // Get command from command line args
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <command> [args...]", args[0]);
        eprintln!("Example: {} echo 'Hello World'", args[0]);
        eprintln!("Example: {} top", args[0]);
        return Ok(());
    }

    let command = &args[1];
    let cmd_args: Vec<&str> = args[2..].iter().map(|s| s.as_str()).collect();

    log::info!("Running command: {} with args: {:?}", command, cmd_args);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create animation
    let terminal_size = terminal.size()?;
    log::info!("Terminal size: {}x{}", terminal_size.width, terminal_size.height);
    
    let rect = Rect::new(0, 0, terminal_size.width, terminal_size.height);
    let mut animation: Option<Animation> = Animation::new(command, &cmd_args, rect);
    
    if animation.is_none() {
        log::error!("Failed to create animation");
        // Cleanup and exit
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        eprintln!("Failed to create animation. Check test_animation.log for details.");
        return Ok(());
    }

    log::info!("Animation created successfully, starting loop");

    let mut last_update = Instant::now();
    let tick_rate = Duration::from_millis(16); // 60 FPS
    let mut first_frame = true;

    loop {
        // Check for exit key
        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    break;
                }
            }
        }

        // Update animation
        let mut needs_redraw = first_frame; // Always redraw first frame
        first_frame = false;
        
        if last_update.elapsed() >= tick_rate {
            if let Some(ref mut anim) = animation {
                if anim.update() {
                    needs_redraw = true;
                    log::debug!("Animation updated");
                }
            }
            last_update = Instant::now();
        }

        // Redraw if needed
        if needs_redraw {
            terminal.draw(|frame| {
                if let Some(ref anim) = animation {
                    frame.render_widget(anim, frame.area());
                }
                
                // Add instructions in bottom-right corner
                let instructions = Paragraph::new("Press 'q' or ESC to quit")
                    .style(Style::default().bg(Color::Black).fg(Color::White));
                let area = frame.area();
                let instruction_area = Rect {
                    x: area.width.saturating_sub(25),
                    y: area.height.saturating_sub(1),
                    width: 25,
                    height: 1,
                };
                frame.render_widget(instructions, instruction_area);
            })?;
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, cursor::Show)?;
    
    log::info!("Test animation exiting");
    println!("Check test_animation.log for debug output");

    Ok(())
} 