use herdr_insight_common::{AgentState, StateTransition};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::config::TimelineConfig;

use super::state::TimelineState;

pub fn draw(frame: &mut Frame, state: &TimelineState, config: &TimelineConfig) {
    let area = frame.area();

    let block = Block::default()
        .title(" Herdr Insight \u{00b7} Timeline ")
        .title_style(Style::default().bold().fg(Color::Cyan))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Gray));
    frame.render_widget(block, area);

    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    if inner.height < 2 {
        return;
    }

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);

    render_entries(frame, rows[0], state, config);
    render_footer(frame, rows[1]);
}

fn render_entries(frame: &mut Frame, area: Rect, state: &TimelineState, config: &TimelineConfig) {
    // Deduplicate by pane_id - keep only the latest entry per agent
    let mut seen_panes = std::collections::HashSet::new();
    let mut entries: Vec<&StateTransition> = Vec::new();
    for t in state.transitions.iter().rev() {
        if seen_panes.insert(&t.pane_id) {
            entries.push(t);
        }
    }

    let visible = area.height as usize;
    let start = state.scroll_offset.min(entries.len().saturating_sub(1));

    // Show header if there is space
    let header_height = if area.height >= 2 { 1u16 } else { 0u16 };
    let entry_area = Rect {
        x: area.x,
        y: area.y + header_height,
        width: area.width,
        height: area.height.saturating_sub(header_height),
    };

    if header_height > 0 {
        render_header(frame, area, config);
    }

    if entries.is_empty() {
        let msg = Paragraph::new(" No agents detected. Waiting for agent activity...")
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(msg, entry_area);
        return;
    }

    let visible_entries = visible.saturating_sub(header_height as usize);
    for (idx, entry) in entries.iter().enumerate().skip(start).take(visible_entries) {
        let y = entry_area.y + (idx - start) as u16;
        if y >= entry_area.y + entry_area.height {
            break;
        }
        let row = Rect::new(entry_area.x, y, entry_area.width, 1);
        render_entry(
            frame,
            row,
            entry,
            config,
            &state.active_panes,
            &state.task_start_times,
        );
    }
}

fn render_header(frame: &mut Frame, area: Rect, config: &TimelineConfig) {
    let row = Rect::new(area.x, area.y, area.width, 1);
    let mut spans = Vec::new();

    // Match data column widths exactly
    if config.columns.time {
        // Data: " HH:MM " = 7 chars
        spans.push(Span::styled(
            " TIME  ",
            Style::default().fg(Color::DarkGray),
        ));
    }
    if config.columns.state {
        // Data: icon (1-2 chars) + " " = 2-3 chars
        spans.push(Span::styled("ST ", Style::default().fg(Color::DarkGray)));
    }
    if config.columns.agent {
        // Data: label (variable) + " "
        spans.push(Span::styled("AGENT ", Style::default().fg(Color::DarkGray)));
    }
    if config.columns.pane {
        // Data: "[wX:pY]" = 8 chars + " " = 9 chars
        spans.push(Span::styled(
            "PANE     ",
            Style::default().fg(Color::DarkGray),
        ));
    }
    if config.columns.status {
        // Data: "{state:<10} " = 11 chars
        spans.push(Span::styled(
            "STATUS     ",
            Style::default().fg(Color::DarkGray),
        ));
    }
    if config.columns.duration {
        // Data: "XXXmin " or "XXh " or "XXs " = 5-6 chars
        spans.push(Span::styled("DUR   ", Style::default().fg(Color::DarkGray)));
    }
    if config.columns.session {
        // Data: full UUID (36 chars) + " " = 37 chars
        spans.push(Span::styled(
            "SESSION                              ",
            Style::default().fg(Color::DarkGray),
        ));
    }

    let line = Line::from(spans);
    frame.render_widget(Paragraph::new(line).style(Style::default()), row);
}

fn render_entry(
    frame: &mut Frame,
    area: Rect,
    entry: &StateTransition,
    config: &TimelineConfig,
    active_panes: &std::collections::HashSet<String>,
    task_start_times: &std::collections::HashMap<String, chrono::DateTime<chrono::Utc>>,
) {
    let mut spans = Vec::new();

    // TIME column
    if config.columns.time {
        let time = entry.timestamp.format("%H:%M").to_string();
        spans.push(Span::styled(
            format!(" {time} "),
            Style::default().fg(Color::DarkGray),
        ));
    }

    // STATE column (icon) - show current state based on active_panes
    if config.columns.state {
        let current_state = if active_panes.contains(&entry.pane_id) {
            entry.to // Agent is still in the same state
        } else {
            herdr_insight_common::AgentState::Idle // Agent is now idle
        };
        let (icon, color) = state_icon_color(current_state);
        spans.push(Span::styled(icon, Style::default().fg(color).bold()));
        spans.push(Span::raw(" "));
    }

    // AGENT column
    if config.columns.agent {
        let label = entry.agent_label.as_deref().unwrap_or("unknown");
        spans.push(Span::styled(label, Style::default().bold()));
        spans.push(Span::raw(" "));
    }

    // PANE column
    if config.columns.pane {
        let short_id = short_pane_id(&entry.pane_id);
        spans.push(Span::styled(
            format!("[{short_id}]"),
            Style::default().fg(Color::DarkGray),
        ));
        spans.push(Span::raw(" "));
    }

    // STATUS column - show current state based on active_panes
    if config.columns.status {
        let current_state = if active_panes.contains(&entry.pane_id) {
            entry.to // Agent is still in the same state
        } else {
            herdr_insight_common::AgentState::Idle // Agent is now idle
        };
        let state_str = format!("{:?}", current_state).to_lowercase();
        spans.push(Span::raw(format!("{state_str:<10} ")));
    }

    // DURATION column - show task duration (stopped if task ended, live if still active)
    if config.columns.duration {
        // Use actual task start time if available, otherwise fall back to entry timestamp
        let start_time = task_start_times
            .get(&entry.pane_id)
            .copied()
            .unwrap_or(entry.timestamp);
        let end_time = entry.ended_at.unwrap_or_else(chrono::Utc::now);
        let dur = (end_time - start_time).num_milliseconds() as f64 / 1000.0;
        if dur >= 3600.0 {
            spans.push(Span::styled(
                format!("{:4.0}h ", dur / 3600.0),
                Style::default().fg(Color::Gray),
            ));
        } else if dur >= 60.0 {
            spans.push(Span::styled(
                format!("{:4.0}min ", dur / 60.0),
                Style::default().fg(Color::Gray),
            ));
        } else {
            spans.push(Span::styled(
                format!("{:3.0}s ", dur),
                Style::default().fg(Color::Gray),
            ));
        }
    }

    // SESSION column
    if config.columns.session {
        let session_display = match &entry.session_id {
            Some(sid) => format!("{sid} "),
            None => "\u{2014} ".to_string(), // em dash
        };
        spans.push(Span::styled(
            session_display,
            Style::default().fg(Color::DarkGray),
        ));
    }

    let line = Line::from(spans);
    frame.render_widget(Paragraph::new(line), area);
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let line = Line::from(Span::styled(
        " q quit  \u{2191}\u{2193} scroll",
        Style::default().fg(Color::DarkGray),
    ));
    frame.render_widget(Paragraph::new(line), area);
}

fn state_icon_color(state: AgentState) -> (&'static str, Color) {
    match state {
        AgentState::Working => ("\u{25cf}", Color::Yellow),
        AgentState::Blocked => ("\u{25b2}", Color::Red),
        AgentState::Done => ("\u{25cb}", Color::Green),
        AgentState::Idle => ("\u{25cc}", Color::Gray),
        AgentState::Unknown => ("?", Color::Gray),
    }
}

/// Extract a compact pane identifier for display.
/// "wB:p1" \u{2192} "wB:p1", "term_654e1721018e91" \u{2192} first 8 chars.
fn short_pane_id(pane_id: &str) -> String {
    // herdr pane IDs like "w7:p4" are already short
    if pane_id.contains(':') {
        pane_id.to_string()
    } else {
        // Terminal IDs are long hashes; truncate
        pane_id.chars().take(8).collect()
    }
}
