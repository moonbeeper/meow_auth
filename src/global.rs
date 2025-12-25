use crate::settings::Settings;

pub struct GlobalState {
    settings: Settings,
}

impl GlobalState {
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }
}
