use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AgentState {
    Idle,
    Working,
    Blocked,
    Done,
    Unknown,
}

impl From<Option<String>> for AgentState {
    fn from(value: Option<String>) -> Self {
        match value.as_deref() {
            Some("idle") => AgentState::Idle,
            Some("working") => AgentState::Working,
            Some("blocked") => AgentState::Blocked,
            Some("done") => AgentState::Done,
            _ => AgentState::Unknown,
        }
    }
}

impl AgentState {
    /// Returns true if this state represents an active task (working or blocked).
    pub fn is_active(&self) -> bool {
        matches!(self, AgentState::Working | AgentState::Blocked)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSnapshot {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub pane_id: String,
    pub agent_label: Option<String>,
    pub state: AgentState,
    pub workspace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pane_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub pane_id: String,
    pub agent_label: Option<String>,
    pub from: AgentState,
    pub to: AgentState,
    pub duration_secs: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_output: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub enum OutputKind {
    Diff,
    Log,
    Error,
    Summary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct OutputEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub pane_id: String,
    pub agent_label: String,
    pub workspace: String,
    pub output_kind: OutputKind,
    pub content: String,
}

// ---- Pane list API types ----

/// Raw herdr pane list response wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaneListResponse {
    pub result: PaneListResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaneListResult {
    pub panes: Vec<PaneInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaneInfo {
    #[serde(rename = "pane_id")]
    pub id: String,
    #[serde(rename = "workspace_id")]
    pub workspace_id: Option<String>,
    #[serde(rename = "tab_id")]
    pub tab_id: Option<String>,
    #[serde(rename = "agent_status", default)]
    pub agent_status: Option<String>,
    /// The detected agent name (e.g. "claude", "codex"). herdr uses the "agent" field.
    #[serde(default, rename = "agent")]
    pub agent_label: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub focused: bool,
    /// Agent session info from herdr. Can be a JSON object with "value" field.
    #[serde(default, rename = "agent_session")]
    pub agent_session: Option<serde_json::Value>,
    /// Derived: workspace name from workspace_id or cwd.
    #[serde(skip)]
    pub workspace_name: Option<String>,
}

impl PaneInfo {
    /// Extract session ID string from agent_session JSON object.
    pub fn session_id(&self) -> Option<String> {
        match &self.agent_session {
            Some(serde_json::Value::Object(map)) => {
                map.get("value").and_then(|v| v.as_str()).map(String::from)
            }
            Some(serde_json::Value::String(s)) => Some(s.clone()),
            _ => None,
        }
    }
}

// ---- Error types ----

use std::io;

#[derive(Debug, thiserror::Error)]
pub enum InsightError {
    #[error("herdr CLI not found at '{0}'")]
    HerdrNotFound(String),

    #[error("herdr CLI call failed: {0}")]
    HerdrCallFailed(#[source] io::Error),

    #[error("invalid JSON from herdr: {0}")]
    InvalidJson(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("data file corrupted: {0}")]
    DataCorrupted(String),

    #[error("terminal setup failed: {0}")]
    Terminal(#[source] io::Error),
}

pub type InsightResult<T> = Result<T, InsightError>;
