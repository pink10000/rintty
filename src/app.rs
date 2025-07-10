#[derive(Debug, PartialEq)]
pub enum ActiveField {
    Username,
    Password,
}

// application state (what gets changed each loop)
pub struct App {
    pub username: String,
    pub password: String,
    pub active_field: ActiveField,
}

impl App {
    pub fn new() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            active_field: ActiveField::Username,
        }
    }
}
