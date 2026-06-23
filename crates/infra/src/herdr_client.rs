use herdr_insight_common::{InsightError, InsightResult, PaneInfo, PaneListResponse};
use serde::de::DeserializeOwned;
use std::process::Command;

/// Returns the herdr binary path from $HERDR_BIN_PATH, or "herdr" as fallback.
pub(crate) fn herdr_bin() -> String {
    std::env::var("HERDR_BIN_PATH").unwrap_or_else(|_| "herdr".to_string())
}

/// Read pane output for preview. Returns last non-empty line, truncated to max_chars.
pub fn read_pane_preview(pane_id: &str, max_chars: usize) -> Option<String> {
    let bin = herdr_bin();
    let output = Command::new(&bin)
        .args(["pane", "read", pane_id, "--source", "visible", "--lines", "5"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let last_line = stdout
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())?;

    let truncated: String = last_line.chars().take(max_chars).collect();
    if truncated.len() < last_line.len() {
        Some(format!("{truncated}..."))
    } else {
        Some(truncated)
    }
}

/// Runs `herdr` with the given args and returns parsed JSON stdout.
pub(crate) fn herdr_json<T: DeserializeOwned>(args: &[&str]) -> InsightResult<T> {
    let bin = herdr_bin();
    let output = Command::new(&bin).args(args).output().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            InsightError::HerdrNotFound(bin.clone())
        } else {
            InsightError::HerdrCallFailed(e)
        }
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(InsightError::HerdrCallFailed(std::io::Error::other(
            format!("herdr exited with {}: {}", output.status, stderr.trim()),
        )));
    }

    serde_json::from_slice(&output.stdout).map_err(InsightError::InvalidJson)
}

/// Call `herdr pane list` (defaults to JSON) and parse.
pub fn list_all_panes() -> InsightResult<Vec<PaneInfo>> {
    let response: PaneListResponse = herdr_json(&["pane", "list"])?;
    Ok(response.result.panes)
}
