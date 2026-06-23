use herdr_insight_common::StateTransition;
use std::collections::{HashMap, HashSet};

use crate::config::TimelineConfig;

pub struct TimelineState {
    pub transitions: Vec<StateTransition>,
    #[allow(dead_code)]
    pub running: bool,
    pub scroll_offset: usize,
    pub error_message: Option<String>,
    pub config: TimelineConfig,
    /// Set of pane IDs that are currently in active state (working/blocked).
    pub active_panes: HashSet<String>,
    /// Track when each pane_id started its current active task.
    pub task_start_times: HashMap<String, chrono::DateTime<chrono::Utc>>,
}

impl TimelineState {
    pub fn new(config: TimelineConfig) -> Self {
        Self {
            transitions: Vec::new(),
            running: true,
            scroll_offset: 0,
            error_message: None,
            config,
            active_panes: HashSet::new(),
            task_start_times: HashMap::new(),
        }
    }
}
