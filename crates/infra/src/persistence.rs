use herdr_insight_common::{InsightError, InsightResult};
use serde::Serialize;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

pub(crate) fn state_dir() -> InsightResult<PathBuf> {
    std::env::var("HERDR_PLUGIN_STATE_DIR")
        .map(PathBuf::from)
        .map_err(|_| InsightError::DataCorrupted("HERDR_PLUGIN_STATE_DIR not set".into()))
}

pub fn append_jsonl_at<T: Serialize>(
    base_dir: &Path,
    filename: &str,
    entry: &T,
) -> InsightResult<()> {
    fs::create_dir_all(base_dir)?;
    let path = base_dir.join(filename);
    let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
    let line = serde_json::to_string(entry)?;
    writeln!(file, "{line}")?;
    Ok(())
}

pub fn append_jsonl<T: Serialize>(filename: &str, entry: &T) -> InsightResult<()> {
    let dir = state_dir()?.join("data");
    append_jsonl_at(&dir, filename, entry)
}

pub fn read_jsonl_at<T: serde::de::DeserializeOwned>(
    base_dir: &Path,
    filename: &str,
) -> InsightResult<Vec<T>> {
    let path = base_dir.join(filename);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file = File::open(&path)?;
    let reader = BufReader::new(file);
    let mut results = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<T>(&line) {
            Ok(item) => results.push(item),
            Err(_) => {
                tracing::warn!("skipping corrupted JSONL line in {}", filename);
            }
        }
    }
    Ok(results)
}

pub fn read_jsonl<T: serde::de::DeserializeOwned>(filename: &str) -> InsightResult<Vec<T>> {
    let dir = state_dir()?.join("data");
    read_jsonl_at(&dir, filename)
}

pub fn purge_old_timeline_entries(retention_days: i64) -> InsightResult<()> {
    use herdr_insight_common::StateTransition;

    let dir = state_dir()?.join("data");
    let path = dir.join("timeline.jsonl");
    if !path.exists() {
        return Ok(());
    }
    let cutoff = chrono::Utc::now() - chrono::Duration::days(retention_days);
    let file = File::open(&path)?;
    let reader = BufReader::new(file);
    let mut kept = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(transition) = serde_json::from_str::<StateTransition>(&line) {
            if transition.timestamp >= cutoff {
                kept.push(line);
            }
        } else {
            kept.push(line);
        }
    }
    let mut file = File::create(&path)?;
    for line in kept {
        writeln!(file, "{line}")?;
    }
    Ok(())
}
