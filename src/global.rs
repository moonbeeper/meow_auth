use std::sync::Arc;

use crate::settings::Settings;

pub struct GlobalState {
    pub settings: Settings,
}

impl GlobalState {
    pub fn new(settings: Settings) -> anyhow::Result<Arc<Self>> {
        Ok(Arc::new(Self { settings }))
    }
}
