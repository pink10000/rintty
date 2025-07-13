use nix::unistd;
use pam;
use std::{ffi::CString, io::Write};
use std::io;
use crossterm::{cursor, execute, terminal};

pub fn authenticate(username: &str, password: &str) -> bool {
    let service = "login";
    let mut auth = pam::Authenticator::with_password(service).unwrap();
    auth.get_handler().set_credentials(username, password);

    auth.authenticate().is_ok() && auth.open_session().is_ok()
}

// TODO: Error handling.
pub fn load_into_shell(username: &str) -> Result<(), io::Error> {
    let mut stdout = io::stdout();
    
    terminal::disable_raw_mode()?; // this allows the terminal to process commands like ctrl-d again
    execute!(
        stdout,
        terminal::LeaveAlternateScreen,
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0),
        cursor::Show
    )?;
    stdout.flush()?;

    let user_info = unistd::User::from_name(username)
        .unwrap()
        .unwrap_or_else(|| panic!("Could not find user {}", username));

    std::env::set_var("USER", username);
    std::env::set_var("LOGNAME", username);
    std::env::set_var("HOME", &user_info.dir);
    std::env::set_var("SHELL", &user_info.shell);
    
    std::env::set_current_dir(&user_info.dir)?;
    
    unistd::setgid(user_info.gid)?; // we should have run this as sudo, so we need to drop root privileges 
    unistd::setuid(user_info.uid)?; // or else we'll log in as root (bad!)
    
    let shell = CString::new(user_info.shell.to_str().unwrap()).unwrap();
    let shell_name = CString::new(
        user_info.shell.file_name().unwrap().to_str().unwrap()
    ).unwrap();
    
    unistd::execv(&shell, &[&shell_name])?;
    Ok(())
}
