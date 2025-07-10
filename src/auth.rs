use nix::unistd;
use pam;
use std::ffi::CString;
use std::io;

pub fn authenticate(username: &str, password: &str) -> bool {
    let service = "login";
    let mut auth = pam::Authenticator::with_password(service).unwrap();
    auth.get_handler().set_credentials(username, password);

    auth.authenticate().is_ok() && auth.open_session().is_ok()
}

// TODO: Error handling.
pub fn load_into_shell(username: &str) -> Result<(), io::Error> {
    let user_info = unistd::User::from_name(username)
        .unwrap()
        .unwrap_or_else(|| panic!("Could not find user {}", username));
    let shell = CString::new(user_info.shell.to_str().unwrap()).unwrap();
    unistd::execv(&shell, &[&shell])?;
    Ok(())
}
