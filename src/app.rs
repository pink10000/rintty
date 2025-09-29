use crate::animation;

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

    pub fn on_tick(&mut self) -> bool {
        if let Some(anim) = &mut self.animation {
            anim.update()
        } else {
            false
        }
    }

    pub fn draw(&mut self, frame: &mut ratatui::Frame, animation_cmd: &Option<String>) {
        if self.animation.is_none() {
            self.animation = animation_cmd.as_ref().map(|cmd| {
                let mut parts = cmd.split_whitespace();
                let command = parts.next().unwrap_or("");
                let args: Vec<&str> = parts.collect();
                log::info!("Creating animation: {} {:?}", command, args);
                let anim = animation::Animation::new(command, &args, frame.area());
                if anim.is_some() {
                    log::info!("Animation created successfully");
                } else {
                    log::error!("Failed to create animation");
                }
                anim
            }).flatten();
        }
        if let Some(anim) = &self.animation {
            frame.render_widget(anim, frame.area());
        }
    }
}
