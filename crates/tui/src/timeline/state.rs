use herdr_insight_common::StateTransition;

use crate::config::TimelineConfig;

pub struct TimelineState {
    pub transitions: Vec<StateTransition>,
    #[allow(dead_code)]
    pub running: bool,
    pub scroll_offset: usize,
    pub error_message: Option<String>,
    pub config: TimelineConfig,
}

impl TimelineState {
    pub fn new(config: TimelineConfig) -> Self {
        Self {
            transitions: Vec::new(),
            running: true,
            scroll_offset: 0,
            error_message: None,
            config,
        }
    }
}
