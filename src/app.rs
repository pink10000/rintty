use crate::animation;
use ratatui::layout::Rect;

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
    pub animation: Option<animation::Animation>,
}

impl App {
    pub fn new() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            active_field: ActiveField::Username,
            animation: None,
        }
    }

    // This method updates the animation state on each tick
    pub fn on_tick(&mut self) {
        if let Some(anim) = &mut self.animation {
            anim.update();
        }
    }

    // Your draw method must render the animation widget first
    pub fn draw(&mut self, frame: &mut ratatui::Frame, animation_cmd: &Option<String>) {
        if self.animation.is_none() {
            self.animation = animation_cmd.as_ref().map(|cmd| {
                let mut parts = cmd.split_whitespace();
                let command = parts.next().unwrap_or("");
                let args: Vec<&str> = parts.collect();
                animation::Animation::new(command, &args, frame.area())
            }).flatten();
        }
        if let Some(anim) = &self.animation {
            frame.render_widget(anim, frame.area());
        }
    }
}
