use herdr_insight_common::{AgentSnapshot, AgentState};
use herdr_insight_domain::{detect_transitions, snapshot_map};
use std::collections::HashMap;

fn make_snap(pane_id: &str, state: AgentState) -> AgentSnapshot {
    AgentSnapshot {
        timestamp: chrono::Utc::now(),
        pane_id: pane_id.to_string(),
        agent_label: Some("test".into()),
        state,
        workspace: "test-ws".into(),
        custom_status: None,
        pane_title: None,
        session_id: None,
        last_output: None,
    }
}

#[test]
fn detect_transition_when_state_changes() {
    let prev = HashMap::from([(
        "pane-1".to_string(),
        make_snap("pane-1", AgentState::Working),
    )]);
    let current = vec![make_snap("pane-1", AgentState::Done)];
    let transitions = detect_transitions(&prev, &current);
    assert_eq!(transitions.len(), 1);
    assert_eq!(transitions[0].from, AgentState::Working);
    assert_eq!(transitions[0].to, AgentState::Done);
    assert_eq!(transitions[0].pane_id, "pane-1");
}

#[test]
fn detect_new_pane_as_transition() {
    let prev = HashMap::new();
    let current = vec![make_snap("pane-new", AgentState::Working)];
    let transitions = detect_transitions(&prev, &current);
    assert_eq!(transitions.len(), 1);
    assert_eq!(transitions[0].from, AgentState::Unknown);
    assert_eq!(transitions[0].to, AgentState::Working);
}

#[test]
fn no_transition_when_state_unchanged() {
    let prev = HashMap::from([(
        "pane-1".to_string(),
        make_snap("pane-1", AgentState::Working),
    )]);
    let current = vec![make_snap("pane-1", AgentState::Working)];
    let transitions = detect_transitions(&prev, &current);
    assert!(transitions.is_empty());
}

#[test]
fn snapshot_map_builds_lookup() {
    let snaps = vec![
        make_snap("pane-1", AgentState::Working),
        make_snap("pane-2", AgentState::Idle),
    ];
    let map = snapshot_map(&snaps);
    assert_eq!(map.len(), 2);
    assert_eq!(map.get("pane-1").unwrap().state, AgentState::Working);
}
