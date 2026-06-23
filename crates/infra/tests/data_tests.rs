use herdr_insight_common::{AgentSnapshot, AgentState};
use herdr_insight_infra::{append_jsonl_at, read_jsonl_at};

#[test]
fn roundtrip_jsonl() {
    let tmp = tempfile::tempdir().expect("tempdir");

    let snap = AgentSnapshot {
        timestamp: chrono::Utc::now(),
        pane_id: "pane-1".into(),
        agent_label: Some("claude".into()),
        state: AgentState::Working,
        workspace: "herdr".into(),
        custom_status: None,
        pane_title: None,
        session_id: None,
        last_output: None,
    };
    append_jsonl_at(tmp.path(), "test.jsonl", &snap).expect("append");
    let read: Vec<AgentSnapshot> = read_jsonl_at(tmp.path(), "test.jsonl").expect("read");
    assert_eq!(read.len(), 1);
    assert_eq!(read[0].pane_id, "pane-1");
}

#[test]
fn read_empty_file_returns_empty_vec() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let result: Vec<AgentSnapshot> = read_jsonl_at(tmp.path(), "nonexistent.jsonl").expect("read");
    assert!(result.is_empty());
}
