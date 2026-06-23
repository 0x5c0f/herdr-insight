mod render;
mod state;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use herdr_insight_common::InsightResult;
use ratatui::prelude::*;
use state::TimelineState;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub fn run() -> InsightResult<()> {
    enable_raw_mode().map_err(herdr_insight_common::InsightError::Terminal)?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, crossterm::cursor::Hide)
        .map_err(herdr_insight_common::InsightError::Terminal)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)
        .map_err(|e| herdr_insight_common::InsightError::Terminal(std::io::Error::other(e)))?;

    let config = crate::config::TimelineConfig::load();
    let mut state = TimelineState::new(config);

    // Load existing timeline from data file
    if let Ok(existing) =
        herdr_insight_infra::read_jsonl::<herdr_insight_common::StateTransition>("timeline.jsonl")
    {
        state.transitions = existing;
    }

    let mut last_poll = Instant::now()
        .checked_sub(Duration::from_secs(3))
        .unwrap_or(Instant::now());
    let mut previous_snapshots = HashMap::new();

    let result = loop {
        let now = Instant::now();
        if now.duration_since(last_poll) >= Duration::from_secs(2) {
            if let Ok(panes) = herdr_insight_infra::list_all_panes() {
                if let Ok(mut snaps) = herdr_insight_domain::poll_snapshots(&panes) {
                    // Enrich snapshots with output preview if configured
                    if state.config.columns.output {
                        for snap in &mut snaps {
                            snap.last_output =
                                herdr_insight_infra::read_pane_preview(&snap.pane_id, 60);
                        }
                    }
                    let transitions =
                        herdr_insight_domain::detect_transitions(&previous_snapshots, &snaps);
                    for t in &transitions {
                        let _ = herdr_insight_infra::append_jsonl("timeline.jsonl", t);
                        state.transitions.push(t.clone());
                    }
                    previous_snapshots = herdr_insight_domain::snapshot_map(&snaps);
                    state.error_message = None;

                    // Purge entries older than 7 days
                    let _ = herdr_insight_infra::purge_old_timeline_entries(7);
                } else {
                    state.error_message = Some("herdr connection lost".to_string());
                }
            } else {
                state.error_message = Some("herdr connection lost".to_string());
            }
            last_poll = now;
        }

        let _ = terminal.draw(|frame| render::draw(frame, &state, &state.config));

        if event::poll(Duration::from_millis(16))
            .map_err(herdr_insight_common::InsightError::Terminal)?
        {
            if let Event::Key(key) =
                event::read().map_err(herdr_insight_common::InsightError::Terminal)?
            {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        break Ok(());
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        // Scroll up toward newer entries (decrease offset)
                        state.scroll_offset = state.scroll_offset.saturating_sub(1);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        // Scroll down toward older entries (increase offset)
                        state.scroll_offset = state.scroll_offset.saturating_add(1);
                    }
                    _ => {}
                }
            }
        }
    };

    disable_raw_mode().map_err(herdr_insight_common::InsightError::Terminal)?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        crossterm::cursor::Show
    )
    .map_err(herdr_insight_common::InsightError::Terminal)?;

    result
}
