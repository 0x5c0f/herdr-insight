pub mod herdr_client;
pub mod persistence;

// Re-export public API for convenience
pub use herdr_client::{list_all_panes, read_pane_preview};
pub use persistence::{
    append_jsonl, append_jsonl_at, purge_old_timeline_entries, read_jsonl, read_jsonl_at,
};
