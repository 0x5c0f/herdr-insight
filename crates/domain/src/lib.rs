pub mod ports;

use herdr_insight_common::{AgentSnapshot, AgentState, InsightResult, PaneInfo, StateTransition};
use std::collections::HashMap;

/// Build agent snapshots from pane info.
///
/// TODO: When more data sources exist (Collector, Dashboard), accept
/// `impl PaneRepository` instead of a plain slice.
pub fn poll_snapshots(pane_infos: &[PaneInfo]) -> InsightResult<Vec<AgentSnapshot>> {
    let now = chrono::Utc::now();
    let snapshots: Vec<AgentSnapshot> = pane_infos
        .iter()
        // Only track panes that have a detected agent (e.g. claude, codex)
        .filter(|p| p.agent_label.is_some())
        .map(|p| AgentSnapshot {
            timestamp: now,
            pane_id: p.id.clone(),
            agent_label: p.agent_label.clone(),
            state: AgentState::from(p.agent_status.clone()),
            workspace: p
                .workspace_name
                .clone()
                .unwrap_or_else(|| p.cwd.clone().unwrap_or_default()),
            custom_status: None,
            pane_title: None,
            session_id: p.session_id(),
            last_output: None,
        })
        .collect();
    Ok(snapshots)
}

/// Detect state transitions by comparing previous and current snapshots.
pub fn detect_transitions(
    previous: &HashMap<String, AgentSnapshot>,
    current: &[AgentSnapshot],
) -> Vec<StateTransition> {
    let now = chrono::Utc::now();
    let mut transitions = Vec::new();

    for snap in current {
        // Skip panes without a detected agent
        if snap.agent_label.is_none() {
            continue;
        }
        let prev = previous.get(&snap.pane_id);
        let state_changed = prev.map(|p| p.state != snap.state).unwrap_or(true);
        if state_changed {
            let duration_secs =
                prev.map(|p| (now - p.timestamp).num_milliseconds() as f64 / 1000.0);
            // Set ended_at when transitioning from active state to inactive
            let ended_at = if prev.map(|p| p.state.is_active()).unwrap_or(false)
                && !snap.state.is_active()
            {
                Some(now)
            } else {
                None
            };
            transitions.push(StateTransition {
                timestamp: now,
                pane_id: snap.pane_id.clone(),
                agent_label: snap.agent_label.clone(),
                from: prev.map(|p| p.state).unwrap_or(AgentState::Unknown),
                to: snap.state,
                duration_secs,
                ended_at,
                session_id: snap.session_id.clone(),
                last_output: snap.last_output.clone(),
            });
        }
    }

    transitions
}

/// Build a lookup map from pane_id to snapshot.
pub fn snapshot_map(snapshots: &[AgentSnapshot]) -> HashMap<String, AgentSnapshot> {
    snapshots
        .iter()
        .cloned()
        .map(|s| (s.pane_id.clone(), s))
        .collect()
}
