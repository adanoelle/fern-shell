//! View rendering for the planner TUI.

use chrono::{Datelike, Local, NaiveDate, Timelike};
use frond::prelude::*;

use crate::data::MonthData;
use crate::model::{DayPane, DetailsFocus, EventFormField, InputMode, MenuAction, MonthPane, PlannerModel, View, WeekPane};

// === Color Constants ===
const COLOR_TODAY: Color = Color::Yellow;
const COLOR_SELECTED: Color = Color::Cyan;
const COLOR_HEADER: Color = Color::Rgb(152, 251, 152); // Pastel green
const COLOR_INVALID_DAY: Color = Color::DarkGray;
const COLOR_HELP: Color = Color::DarkGray;

// Month view colors
const COLOR_EVENT_DOT: Color = Color::Rgb(180, 180, 220); // Soft lavender
const COLOR_FOCUSED_BORDER: Color = Color::Cyan;
const COLOR_UNFOCUSED_BORDER: Color = Color::DarkGray;
const COLOR_SECTION_HEADER: Color = Color::Rgb(152, 251, 152); // Pastel green
const COLOR_GOAL_COMPLETED: Color = Color::Green;
const COLOR_GOAL_PENDING: Color = Color::White;
const COLOR_WEEKDAY: Color = Color::White;
const COLOR_WEEKEND: Color = Color::Rgb(255, 182, 193); // Light pink
const COLOR_CURRENT_HOUR: Color = Color::Yellow; // Highlight for current hour
const COLOR_HOUR_EMPTY: Color = Color::Rgb(60, 65, 75); // Dim color for empty hour slots
const COLOR_EVENT_BLOCK: Color = Color::Rgb(100, 140, 200); // Blue-ish for event blocks

/// Main render function - dispatches to the appropriate view.
pub fn render(model: &PlannerModel, frame: &mut Frame) {
    match model.view {
        View::Year => render_year_view(model, frame),
        View::Month => render_month_view(model, frame),
        View::Week => render_week_view(model, frame),
        View::Day => render_day_view(model, frame),
    }
}

/// Render the year view - months as columns, days as rows.
fn render_year_view(model: &PlannerModel, frame: &mut Frame) {
    let area = frame.area();

    // Layout: calendar grid with border, help at bottom
    let chunks = Layout::vertical([
        Constraint::Min(0),    // Calendar grid with border
        Constraint::Length(1), // Help line
    ])
    .split(area);

    // Block with rounded corners and year title (top left)
    let title = year_to_words(model.view_year);
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let inner_area = block.inner(chunks[0]);
    frame.render_widget(block, chunks[0]);

    // Calendar grid inside the block
    render_year_grid(model, frame, inner_area);

    // Help line
    let help = "h/l: month │ j/k: day │ J/K: week │ H/L: year │ [n]movement │ t: today │ q: quit";
    let help_paragraph = Paragraph::new(help)
        .style(Style::default().fg(COLOR_HELP))
        .alignment(Alignment::Center);
    frame.render_widget(help_paragraph, chunks[1]);
}

/// Render the year grid with months as columns and days as rows.
fn render_year_grid(model: &PlannerModel, frame: &mut Frame, area: Rect) {
    // 12 columns for months (no day number column)
    let col_width = area.width / 12;

    // Small fixed margin from top of box
    let top_margin: u16 = 1;

    // Header row with month abbreviations - bold pastel green
    let month_names = ["JAN", "FEB", "MAR", "APR", "MAY", "JUN",
                       "JUL", "AUG", "SEP", "OCT", "NOV", "DEC"];
    let mut header_spans = vec![];
    for name in month_names {
        let month_str = format!("{:^width$}", name, width = col_width as usize);
        header_spans.push(Span::styled(
            month_str,
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(COLOR_HEADER),
        ));
    }
    let header_line = Line::from(header_spans);
    let header = Paragraph::new(header_line);
    frame.render_widget(header, Rect::new(area.x, area.y + top_margin, area.width, 1));

    // Day rows (1-31)
    let grid_area = Rect::new(area.x, area.y + top_margin + 1, area.width, 31.min(area.height.saturating_sub(top_margin + 1)));

    let mut lines: Vec<Line> = Vec::with_capacity(31);

    for day in 1..=31 {
        let mut spans = vec![];

        // Each month column (no day number column)
        for month in 1..=12u32 {
            let cell = format_day_cell(model, model.view_year, month, day, col_width as usize);
            spans.push(cell);
        }

        lines.push(Line::from(spans));
    }

    let grid = Paragraph::new(lines);
    frame.render_widget(grid, grid_area);
}

/// Format a single day cell in the year grid.
fn format_day_cell(model: &PlannerModel, year: i32, month: u32, day: u32, width: usize) -> Span<'static> {
    // Check if this day exists in this month
    let date = NaiveDate::from_ymd_opt(year, month, day);

    match date {
        Some(date) => {
            let is_today = date == model.today;
            let is_selected = date == model.selected_date;

            let day_str = if is_today && is_selected {
                format!("{:^width$}", format!("[{:02}]", day), width = width)
            } else if is_today {
                format!("{:^width$}", format!("({:02})", day), width = width)
            } else if is_selected {
                format!("{:^width$}", format!("[{:02}]", day), width = width)
            } else {
                format!("{:^width$}", format!("{:02}", day), width = width)
            };

            let style = if is_today {
                Style::default()
                    .fg(COLOR_TODAY)
                    .add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default()
                    .fg(COLOR_SELECTED)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            Span::styled(day_str, style)
        }
        None => {
            // Invalid day for this month (e.g., Feb 30)
            Span::styled(
                format!("{:^width$}", "·", width = width),
                Style::default().fg(COLOR_INVALID_DAY),
            )
        }
    }
}

/// Convert a year number to words (e.g., 2026 -> "TWENTY TWENTY SIX").
fn year_to_words(year: i32) -> String {
    let ones = [
        "", "ONE", "TWO", "THREE", "FOUR", "FIVE", "SIX", "SEVEN", "EIGHT", "NINE",
    ];
    let teens = [
        "TEN", "ELEVEN", "TWELVE", "THIRTEEN", "FOURTEEN", "FIFTEEN",
        "SIXTEEN", "SEVENTEEN", "EIGHTEEN", "NINETEEN",
    ];
    let tens = [
        "", "", "TWENTY", "THIRTY", "FORTY", "FIFTY", "SIXTY", "SEVENTY", "EIGHTY", "NINETY",
    ];

    let num_to_words = |n: i32| -> String {
        if n == 0 {
            return String::new();
        }
        let n = n as usize;
        if n < 10 {
            ones[n].to_string()
        } else if n < 20 {
            teens[n - 10].to_string()
        } else {
            let t = tens[n / 10];
            let o = ones[n % 10];
            if o.is_empty() {
                t.to_string()
            } else {
                format!("{} {}", t, o)
            }
        }
    };

    // Split year into two parts: first two digits and last two digits
    let first = year / 100;
    let second = year % 100;

    let first_words = num_to_words(first);
    let second_words = num_to_words(second);

    if second_words.is_empty() {
        format!("{} HUNDRED", first_words)
    } else {
        format!("{} {}", first_words, second_words)
    }
}

/// Render the month view with hybrid layout.
fn render_month_view(model: &PlannerModel, frame: &mut Frame) {
    let area = frame.area();

    // Main layout: days pane (40%) | details pane (60%)
    let main_chunks = Layout::horizontal([
        Constraint::Percentage(40),
        Constraint::Percentage(60),
    ])
    .split(area);

    // Get month data (we need to work around the borrow checker)
    let year = model.selected_date.year();
    let month = model.selected_date.month();
    let month_data = model
        .month_data
        .get(&(year, month))
        .cloned()
        .unwrap_or_else(|| crate::data::sample_data(year, month));

    // Render both panes
    render_days_pane(model, frame, main_chunks[0], &month_data);
    render_details_pane(model, frame, main_chunks[1], &month_data);

    // Render popup overlay if in input mode
    match model.input_mode {
        InputMode::Normal => {}
        InputMode::ContextMenu => render_context_menu(model, frame, area),
        InputMode::ConfirmDelete => render_confirm_delete(model, frame, area),
        InputMode::AddingEvent | InputMode::EditingEvent => {
            render_event_form(model, frame, area)
        }
        InputMode::AddingGoal
        | InputMode::AddingIntention
        | InputMode::EditingGoal
        | InputMode::EditingIntention
        | InputMode::AddingWeekNote
        | InputMode::EditingWeekNote => render_popup(model, frame, area),
        InputMode::ViewingWeekNote => {} // Not used in month view
    }
}

/// Render the days pane (left side).
fn render_days_pane(model: &PlannerModel, frame: &mut Frame, area: Rect, month_data: &MonthData) {
    let is_focused = model.month_pane == MonthPane::Days;

    // Layout: content + help line
    let chunks = Layout::vertical([
        Constraint::Min(0),    // Day list
        Constraint::Length(1), // Help line
    ])
    .split(area);

    // Title with month name
    let month_name = month_name(model.selected_date.month()).to_uppercase();
    let title = format!(" {} {} ", month_name, model.selected_date.year());

    let border_color = if is_focused {
        COLOR_FOCUSED_BORDER
    } else {
        COLOR_UNFOCUSED_BORDER
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    let inner_area = block.inner(chunks[0]);
    frame.render_widget(block, chunks[0]);

    // Render day list
    render_day_list(model, frame, inner_area, month_data);

    // Help line
    let help = if is_focused {
        "j/k: day │ h/l: month │ Tab: details │ Enter: day │ Esc: year"
    } else {
        ""
    };
    let help_paragraph = Paragraph::new(help)
        .style(Style::default().fg(COLOR_HELP))
        .alignment(Alignment::Center);
    frame.render_widget(help_paragraph, chunks[1]);
}

/// Render the list of days in the month.
fn render_day_list(model: &PlannerModel, frame: &mut Frame, area: Rect, month_data: &MonthData) {
    let year = model.selected_date.year();
    let month = model.selected_date.month();
    let selected_day = model.selected_date.day();
    let today = model.today;

    let days_in_month = days_in_month(year, month);
    let mut lines: Vec<Line> = Vec::with_capacity(days_in_month as usize);

    for day in 1..=days_in_month {
        let date = NaiveDate::from_ymd_opt(year, month, day).unwrap();
        let is_today = date == today;
        let is_selected = day == selected_day;
        let weekday = date.weekday();
        let weekday_abbr = weekday_abbr(weekday);
        let is_weekend = matches!(weekday, chrono::Weekday::Sat | chrono::Weekday::Sun);

        // Event dots (max 4)
        let event_count = month_data.event_count(day);
        let dots = "·".repeat(event_count.min(4));
        let dots_padding = " ".repeat(4 - event_count.min(4));

        // Format: [15] Mon ····  or  15  Mon ····
        let day_str = if is_today && is_selected {
            format!("[{:02}]", day)
        } else if is_today {
            format!("({:02})", day)
        } else if is_selected {
            format!("[{:02}]", day)
        } else {
            format!(" {:02} ", day)
        };

        let day_style = if is_today {
            Style::default().fg(COLOR_TODAY).add_modifier(Modifier::BOLD)
        } else if is_selected {
            Style::default().fg(COLOR_SELECTED).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let weekday_style = if is_weekend {
            Style::default().fg(COLOR_WEEKEND)
        } else {
            Style::default().fg(COLOR_WEEKDAY)
        };

        let dots_style = Style::default().fg(COLOR_EVENT_DOT);

        let line = Line::from(vec![
            Span::styled(day_str, day_style),
            Span::raw(" "),
            Span::styled(weekday_abbr.to_string(), weekday_style),
            Span::raw(" "),
            Span::styled(dots, dots_style),
            Span::raw(dots_padding),
        ]);

        lines.push(line);
    }

    // Scroll to keep selected day visible
    let scroll_offset = if selected_day > area.height as u32 {
        (selected_day - area.height as u32) as u16
    } else {
        0
    };

    let paragraph = Paragraph::new(lines).scroll((scroll_offset, 0));
    frame.render_widget(paragraph, area);
}

/// Render the details pane (right side).
fn render_details_pane(model: &PlannerModel, frame: &mut Frame, area: Rect, month_data: &MonthData) {
    let is_focused = model.month_pane == MonthPane::Details;

    // Layout: content + help line
    let chunks = Layout::vertical([
        Constraint::Min(0),    // Details content
        Constraint::Length(1), // Help line
    ])
    .split(area);

    // Title with selected date
    let date_str = model.selected_date.format("%B %e, %Y").to_string();
    let title = format!(" {} ", date_str);

    let border_color = if is_focused {
        COLOR_FOCUSED_BORDER
    } else {
        COLOR_UNFOCUSED_BORDER
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    let inner_area = block.inner(chunks[0]);
    frame.render_widget(block, chunks[0]);

    // Split inner area into sections: 50% events, 30% goals, 20% intention
    let section_chunks = Layout::vertical([
        Constraint::Percentage(50),
        Constraint::Percentage(30),
        Constraint::Percentage(20),
    ])
    .split(inner_area);

    render_events_section(model, frame, section_chunks[0], month_data);
    render_goals_section(model, frame, section_chunks[1], month_data);
    render_intention_section(model, frame, section_chunks[2], month_data);

    // Help line
    let help = if is_focused {
        "j/k: item │ g/G: section │ Tab: days │ q: quit"
    } else {
        ""
    };
    let help_paragraph = Paragraph::new(help)
        .style(Style::default().fg(COLOR_HELP))
        .alignment(Alignment::Center);
    frame.render_widget(help_paragraph, chunks[1]);
}

/// Render the events section.
fn render_events_section(model: &PlannerModel, frame: &mut Frame, area: Rect, month_data: &MonthData) {
    let is_focused = model.month_pane == MonthPane::Details
        && model.details_focus == DetailsFocus::Events;

    let border_color = if is_focused {
        COLOR_SECTION_HEADER
    } else {
        COLOR_UNFOCUSED_BORDER
    };

    let block = Block::default()
        .title(" Events ")
        .title_style(Style::default().fg(COLOR_SECTION_HEADER).add_modifier(Modifier::BOLD))
        .borders(Borders::TOP)
        .border_style(Style::default().fg(border_color));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let day = model.selected_date.day();
    let events = month_data.events_for_day(day);

    if events.is_empty() {
        let empty = Paragraph::new("  No events")
            .style(Style::default().fg(COLOR_INVALID_DAY));
        frame.render_widget(empty, inner_area);
        return;
    }

    let mut lines: Vec<Line> = Vec::with_capacity(events.len() * 2);

    for (idx, event) in events.iter().enumerate() {
        let is_selected = is_focused && idx == model.selected_event_idx;
        let prefix = if is_selected { " > " } else { "   " };

        // Time range or "all-day"
        let time_str = format_event_time(event);

        let title_style = if is_selected {
            Style::default().fg(COLOR_SELECTED).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let line = Line::from(vec![
            Span::raw(prefix),
            Span::styled(format!("{:12} ", time_str), Style::default().fg(COLOR_HELP)),
            Span::styled(&event.title, title_style),
        ]);
        lines.push(line);

        // Description on next line if selected
        if is_selected {
            if let Some(ref desc) = event.description {
                let desc_line = Line::from(vec![
                    Span::raw("               "),
                    Span::styled(desc.as_str(), Style::default().fg(COLOR_HELP)),
                ]);
                lines.push(desc_line);
            }
        }
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner_area);
}

/// Render the goals section.
fn render_goals_section(model: &PlannerModel, frame: &mut Frame, area: Rect, month_data: &MonthData) {
    let is_focused = model.month_pane == MonthPane::Details
        && model.details_focus == DetailsFocus::Goals;

    let border_color = if is_focused {
        COLOR_SECTION_HEADER
    } else {
        COLOR_UNFOCUSED_BORDER
    };

    let block = Block::default()
        .title(" Goals ")
        .title_style(Style::default().fg(COLOR_SECTION_HEADER).add_modifier(Modifier::BOLD))
        .borders(Borders::TOP)
        .border_style(Style::default().fg(border_color));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    if month_data.goals.is_empty() {
        let empty = Paragraph::new("  No goals")
            .style(Style::default().fg(COLOR_INVALID_DAY));
        frame.render_widget(empty, inner_area);
        return;
    }

    let mut lines: Vec<Line> = Vec::with_capacity(month_data.goals.len());

    for (idx, goal) in month_data.goals.iter().enumerate() {
        let is_selected = is_focused && idx == model.selected_goal_idx;
        let prefix = if is_selected { " > " } else { "   " };

        let checkbox = if goal.completed { "[x]" } else { "[ ]" };
        let checkbox_color = if goal.completed {
            COLOR_GOAL_COMPLETED
        } else {
            COLOR_GOAL_PENDING
        };

        let title_style = if is_selected {
            Style::default().fg(COLOR_SELECTED).add_modifier(Modifier::BOLD)
        } else if goal.completed {
            Style::default().fg(COLOR_HELP)
        } else {
            Style::default()
        };

        let line = Line::from(vec![
            Span::raw(prefix),
            Span::styled(checkbox, Style::default().fg(checkbox_color)),
            Span::raw(" "),
            Span::styled(&goal.title, title_style),
        ]);
        lines.push(line);
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner_area);
}

/// Render the intentions section (bulleted list).
fn render_intention_section(model: &PlannerModel, frame: &mut Frame, area: Rect, month_data: &MonthData) {
    let is_focused = model.month_pane == MonthPane::Details
        && model.details_focus == DetailsFocus::Intention;

    let border_color = if is_focused {
        COLOR_SECTION_HEADER
    } else {
        COLOR_UNFOCUSED_BORDER
    };

    let block = Block::default()
        .title(" Intentions ")
        .title_style(Style::default().fg(COLOR_SECTION_HEADER).add_modifier(Modifier::BOLD))
        .borders(Borders::TOP)
        .border_style(Style::default().fg(border_color));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    if month_data.intentions.is_empty() {
        let empty = Paragraph::new("  No intentions set")
            .style(Style::default().fg(COLOR_INVALID_DAY));
        frame.render_widget(empty, inner_area);
        return;
    }

    let mut lines: Vec<Line> = Vec::with_capacity(month_data.intentions.len());

    for (idx, intention) in month_data.intentions.iter().enumerate() {
        let is_selected = is_focused && idx == model.selected_intention_idx;
        let prefix = if is_selected { " > " } else { "   " };

        let text_style = if is_selected {
            Style::default().fg(COLOR_SELECTED).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let line = Line::from(vec![
            Span::raw(prefix),
            Span::styled("• ", Style::default().fg(COLOR_EVENT_DOT)),
            Span::styled(&intention.text, text_style),
        ]);
        lines.push(line);
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner_area);
}

/// Format time digits as a masked HH:MM display with cursor.
fn format_time_mask(digits: &str, active: bool) -> String {
    let mut result = ['H', 'H', ':', 'M', 'M'];

    // Overlay typed digits onto template
    for (i, ch) in digits.chars().enumerate() {
        let pos = match i {
            0 => 0, // First H
            1 => 1, // Second H
            2 => 3, // First M (skip colon)
            3 => 4, // Second M
            _ => break,
        };
        result[pos] = ch;
    }

    // Show cursor at next input position when active
    if active && digits.len() < 4 {
        let cursor_pos = match digits.len() {
            0 => 0,
            1 => 1,
            2 => 3,
            3 => 4,
            _ => 4,
        };
        result[cursor_pos] = '_';
    }

    result.iter().collect()
}

/// Format a NaiveTime in 12-hour format with AM/PM.
/// Returns format like "9:00a" or "12:30p".
fn format_time_12h(time: chrono::NaiveTime) -> String {
    let hour = time.hour();
    let minute = time.minute();
    let (h12, period) = if hour == 0 {
        (12, 'a')
    } else if hour < 12 {
        (hour, 'a')
    } else if hour == 12 {
        (12, 'p')
    } else {
        (hour - 12, 'p')
    };
    format!("{}:{:02}{}", h12, minute, period)
}

/// Format an event's time range for display.
/// Returns "all-day" for all-day events, or "9:00a" or "9:00a-10:00a" for timed events.
fn format_event_time(event: &crate::data::Event) -> String {
    match (event.start_time, event.end_time) {
        (None, _) => "all-day".to_string(),
        (Some(start), None) => format_time_12h(start),
        (Some(start), Some(end)) => format!("{}-{}", format_time_12h(start), format_time_12h(end)),
    }
}

/// Render the context menu popup for add/edit/delete actions.
fn render_context_menu(model: &PlannerModel, frame: &mut Frame, area: Rect) {
    // Subtle dim effect
    let dim_style = Style::default().fg(Color::DarkGray).bg(Color::Rgb(28, 31, 38));
    let dim_block = Block::default().style(dim_style);
    frame.render_widget(dim_block, area);

    // Large popup - centered at intersection of panes
    let popup_width: u16 = 56;
    let popup_height: u16 = 19;

    // Center over the full screen area
    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // Clear the area behind the popup
    frame.render_widget(Clear, popup_area);

    // Popup background color
    let popup_bg = Color::Rgb(40, 44, 52);

    // Title based on section
    let section_name = match model.details_focus {
        DetailsFocus::Events => "Event",
        DetailsFocus::Goals => "Goal",
        DetailsFocus::Intention => "Intention",
    };
    let title = format!(" {} Actions ", section_name);

    // Help text at bottom
    let help_text = " j/k: navigate │ Enter: select │ Esc: cancel ";

    let block = Block::default()
        .title(title)
        .title_style(Style::default().fg(COLOR_SECTION_HEADER).add_modifier(Modifier::BOLD))
        .title_bottom(Line::from(help_text).centered())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(COLOR_FOCUSED_BORDER))
        .style(Style::default().bg(popup_bg));

    let inner_area = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    // Check if there's a selected item (for enabling edit/delete)
    let has_item = model.has_selected_item();

    // Menu options with descriptions
    let options = [
        (MenuAction::Add, "Add", "Create a new item", true),
        (MenuAction::Edit, "Edit", "Modify selected item", has_item),
        (MenuAction::Delete, "Delete", "Remove selected item", has_item),
    ];

    // Render each option with spacing
    for (i, (action, label, description, enabled)) in options.iter().enumerate() {
        let is_selected = model.menu_action == *action;
        let y = inner_area.y + 2 + (i as u16 * 5); // Extra vertical spacing

        // Main label
        let label_style = if !enabled {
            Style::default().fg(Color::DarkGray)
        } else if is_selected {
            Style::default().fg(COLOR_SELECTED).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let prefix = if is_selected { "  › " } else { "    " };
        let text = format!("{}{}", prefix, label);
        let para = Paragraph::new(text).style(label_style);
        frame.render_widget(para, Rect::new(inner_area.x, y, inner_area.width, 1));

        // Description on next line (dimmer)
        let desc_style = if !enabled {
            Style::default().fg(Color::Rgb(50, 50, 50))
        } else if is_selected {
            Style::default().fg(COLOR_HELP)
        } else {
            Style::default().fg(Color::Rgb(80, 80, 80))
        };
        let desc_text = format!("      {}", description);
        let desc_para = Paragraph::new(desc_text).style(desc_style);
        frame.render_widget(desc_para, Rect::new(inner_area.x, y + 1, inner_area.width, 1));
    }
}

/// Render the confirm delete dialog.
fn render_confirm_delete(model: &PlannerModel, frame: &mut Frame, area: Rect) {
    // Subtle dim effect
    let dim_style = Style::default().fg(Color::DarkGray).bg(Color::Rgb(28, 31, 38));
    let dim_block = Block::default().style(dim_style);
    frame.render_widget(dim_block, area);

    // Small popup with room for padding
    let popup_width: u16 = 34;
    let popup_height: u16 = 7;

    // Center over the full screen area
    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // Clear the area behind the popup
    frame.render_widget(Clear, popup_area);

    // Popup background color
    let popup_bg = Color::Rgb(40, 44, 52);

    // Get the item name being deleted
    let item_type = match model.details_focus {
        DetailsFocus::Events => "event",
        DetailsFocus::Goals => "goal",
        DetailsFocus::Intention => "intention",
    };

    let block = Block::default()
        .title(" Confirm Delete ")
        .title_style(Style::default().fg(Color::Rgb(255, 100, 100)).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(255, 100, 100)))
        .style(Style::default().bg(popup_bg));

    let inner_area = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    // Message - centered with top margin
    let message = format!("Delete this {}?", item_type);
    let msg_para = Paragraph::new(message)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center);
    frame.render_widget(msg_para, Rect::new(inner_area.x, inner_area.y + 1, inner_area.width, 1));

    // Options with styled keys
    let options = Line::from(vec![
        Span::styled("y", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::styled(": Yes    ", Style::default().fg(COLOR_HELP)),
        Span::styled("n", Style::default().fg(Color::Rgb(255, 100, 100)).add_modifier(Modifier::BOLD)),
        Span::styled(": No", Style::default().fg(COLOR_HELP)),
    ]);
    let opts_para = Paragraph::new(options).alignment(Alignment::Center);
    frame.render_widget(opts_para, Rect::new(inner_area.x, inner_area.y + 3, inner_area.width, 1));
}

/// Render the event form popup for adding events.
fn render_event_form(model: &PlannerModel, frame: &mut Frame, area: Rect) {
    // Subtle dim effect
    let dim_style = Style::default().fg(Color::DarkGray).bg(Color::Rgb(28, 31, 38));
    let dim_block = Block::default().style(dim_style);
    frame.render_widget(dim_block, area);

    // Popup dimensions - 50% width, 14 lines tall (extra row for end time)
    let popup_width = (area.width * 50 / 100).max(50).min(area.width.saturating_sub(4));
    let popup_height: u16 = 14;

    // Center over the full screen area
    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // Clear the area behind the popup
    frame.render_widget(Clear, popup_area);

    // Popup background color
    let popup_bg = Color::Rgb(40, 44, 52);

    // Help text
    let help_text = " Tab: next │ Shift+Tab: prev │ Enter: save │ Esc: cancel ";

    // Title depends on whether we're adding or editing
    let title = if model.input_mode == InputMode::EditingEvent {
        " Edit Event "
    } else {
        " Add Event "
    };

    let block = Block::default()
        .title(title)
        .title_style(Style::default().fg(COLOR_SECTION_HEADER).add_modifier(Modifier::BOLD))
        .title_bottom(Line::from(help_text).centered())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(COLOR_FOCUSED_BORDER))
        .style(Style::default().bg(popup_bg));

    let inner_area = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    // Layout constants
    let label_width: u16 = 16;
    let field_x = inner_area.x;
    let value_x = inner_area.x + label_width;
    let value_width = inner_area.width.saturating_sub(label_width);

    // Styles
    let active_style = Style::default().fg(COLOR_SELECTED);
    let inactive_style = Style::default().fg(COLOR_HELP);
    let label_style = Style::default().fg(Color::White);
    let placeholder_style = Style::default().fg(Color::DarkGray);

    // === Row 0: Title field ===
    let title_y = inner_area.y + 1; // Add top margin
    let is_title_active = model.event_form_field == EventFormField::Title;

    let title_label = Paragraph::new("    Title:")
        .style(if is_title_active { active_style } else { label_style });
    frame.render_widget(title_label, Rect::new(field_x, title_y, label_width, 1));

    let title_value = if is_title_active {
        format!("› {}_", model.event_title)
    } else if model.event_title.is_empty() {
        "  (required)".to_string()
    } else {
        format!("  {}", model.event_title)
    };
    let title_style = if is_title_active {
        active_style
    } else if model.event_title.is_empty() {
        placeholder_style
    } else {
        inactive_style
    };
    let title_para = Paragraph::new(title_value).style(title_style);
    frame.render_widget(title_para, Rect::new(value_x, title_y, value_width, 1));

    // === Row 1: All-day toggle ===
    let allday_y = title_y + 2;
    let is_allday_active = model.event_form_field == EventFormField::AllDay;

    let allday_label = Paragraph::new("    All-day:")
        .style(if is_allday_active { active_style } else { label_style });
    frame.render_widget(allday_label, Rect::new(field_x, allday_y, label_width, 1));

    let checkbox = if model.event_all_day { "[×]" } else { "[ ]" };
    let allday_value = if is_allday_active {
        format!("› {} Space to toggle", checkbox)
    } else {
        format!("  {}", checkbox)
    };
    let allday_para = Paragraph::new(allday_value)
        .style(if is_allday_active { active_style } else { inactive_style });
    frame.render_widget(allday_para, Rect::new(value_x, allday_y, value_width, 1));

    // === Row 2: Start Time + AM/PM ===
    let start_time_y = title_y + 4;
    let is_start_time_active = model.event_form_field == EventFormField::StartTime;
    let is_start_ampm_active = model.event_form_field == EventFormField::StartAmPm;
    let time_disabled = model.event_all_day;

    let start_label_style = if time_disabled {
        inactive_style
    } else if is_start_time_active || is_start_ampm_active {
        active_style
    } else {
        label_style
    };
    let start_time_label = Paragraph::new("    Start:").style(start_label_style);
    frame.render_widget(start_time_label, Rect::new(field_x, start_time_y, label_width, 1));

    // Consistent layout: time at offset 0-9, AM/PM at offset 10
    let ampm_offset: u16 = 10;

    if time_disabled {
        let disabled_text = Paragraph::new("  (all day)").style(placeholder_style);
        frame.render_widget(disabled_text, Rect::new(value_x, start_time_y, value_width, 1));
    } else {
        // Time portion
        let masked = format_time_mask(&model.event_start_time, is_start_time_active);
        let time_prefix = if is_start_time_active { "› " } else { "  " };
        let time_text = format!("{}{}", time_prefix, masked);
        let time_style = if is_start_time_active { active_style } else { inactive_style };
        let time_para = Paragraph::new(time_text).style(time_style);
        frame.render_widget(time_para, Rect::new(value_x, start_time_y, ampm_offset, 1));

        // AM/PM toggle
        let ampm_text = if model.event_start_am { "AM" } else { "PM" };
        let ampm_display = if is_start_ampm_active {
            format!("› {} ◀", ampm_text)
        } else {
            ampm_text.to_string()
        };
        let ampm_style = if is_start_ampm_active { active_style } else { inactive_style };
        let ampm_para = Paragraph::new(ampm_display).style(ampm_style);
        frame.render_widget(ampm_para, Rect::new(value_x + ampm_offset, start_time_y, 8, 1));
    }

    // === Row 3: End Time + AM/PM ===
    let end_time_y = title_y + 6;
    let is_end_time_active = model.event_form_field == EventFormField::EndTime;
    let is_end_ampm_active = model.event_form_field == EventFormField::EndAmPm;

    let end_label_style = if time_disabled {
        inactive_style
    } else if is_end_time_active || is_end_ampm_active {
        active_style
    } else {
        label_style
    };
    let end_time_label = Paragraph::new("    End:").style(end_label_style);
    frame.render_widget(end_time_label, Rect::new(field_x, end_time_y, label_width, 1));

    if time_disabled {
        let disabled_text = Paragraph::new("  (all day)").style(placeholder_style);
        frame.render_widget(disabled_text, Rect::new(value_x, end_time_y, value_width, 1));
    } else {
        // Time portion
        let masked = format_time_mask(&model.event_end_time, is_end_time_active);
        let time_prefix = if is_end_time_active { "› " } else { "  " };
        let time_text = format!("{}{}", time_prefix, masked);
        let time_style = if is_end_time_active { active_style } else { inactive_style };
        let time_para = Paragraph::new(time_text).style(time_style);
        frame.render_widget(time_para, Rect::new(value_x, end_time_y, ampm_offset, 1));

        // AM/PM toggle
        let ampm_text = if model.event_end_am { "AM" } else { "PM" };
        let ampm_display = if is_end_ampm_active {
            format!("› {} ◀", ampm_text)
        } else {
            ampm_text.to_string()
        };
        let ampm_style = if is_end_ampm_active { active_style } else { inactive_style };
        let ampm_para = Paragraph::new(ampm_display).style(ampm_style);
        frame.render_widget(ampm_para, Rect::new(value_x + ampm_offset, end_time_y, 8, 1));
    }

    // === Row 4: Description field ===
    let desc_y = title_y + 8;
    let is_desc_active = model.event_form_field == EventFormField::Description;

    let desc_label = Paragraph::new("    Description:")
        .style(if is_desc_active { active_style } else { label_style });
    frame.render_widget(desc_label, Rect::new(field_x, desc_y, label_width, 1));

    let desc_value = if is_desc_active {
        format!("› {}_", model.event_description)
    } else if model.event_description.is_empty() {
        "  (optional)".to_string()
    } else {
        format!("  {}", model.event_description)
    };
    let desc_style = if is_desc_active {
        active_style
    } else if model.event_description.is_empty() {
        placeholder_style
    } else {
        inactive_style
    };
    let desc_para = Paragraph::new(desc_value).style(desc_style);
    frame.render_widget(desc_para, Rect::new(value_x, desc_y, value_width, 1));
}

/// Render the popup overlay for adding goals/intentions.
fn render_popup(model: &PlannerModel, frame: &mut Frame, area: Rect) {
    // Subtle dim effect - slightly darker than normal background
    let dim_style = Style::default().fg(Color::DarkGray).bg(Color::Rgb(28, 31, 38));
    let dim_block = Block::default().style(dim_style);
    frame.render_widget(dim_block, area);

    // Square-ish popup - 40% width, 9 lines tall
    let popup_width = (area.width * 40 / 100).max(40).min(area.width.saturating_sub(4));
    let popup_height: u16 = 9;

    // Center over the full screen area
    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // Clear the area behind the popup
    frame.render_widget(Clear, popup_area);

    // Title based on input mode (Goal/Intention/WeekNote use this popup)
    let title = match model.input_mode {
        InputMode::AddingGoal => " Add Goal ",
        InputMode::EditingGoal => " Edit Goal ",
        InputMode::AddingIntention => " Add Intention ",
        InputMode::EditingIntention => " Edit Intention ",
        InputMode::AddingWeekNote => " Add Note ",
        InputMode::EditingWeekNote => " Edit Note ",
        InputMode::Normal | InputMode::ContextMenu | InputMode::ConfirmDelete | InputMode::AddingEvent | InputMode::EditingEvent | InputMode::ViewingWeekNote => "",
    };

    // Help text as bottom title
    let help_text = " Enter: submit │ Esc: cancel ";

    // Popup background color for visible rounded corners
    let popup_bg = Color::Rgb(40, 44, 52);

    let block = Block::default()
        .title(title)
        .title_style(Style::default().fg(COLOR_SECTION_HEADER).add_modifier(Modifier::BOLD))
        .title_bottom(Line::from(help_text).centered())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(COLOR_FOCUSED_BORDER))
        .style(Style::default().bg(popup_bg));

    let inner_area = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    // Input field with cursor - positioned at top of inner area
    let input_text = format!("> {}_", model.input_buffer);
    let input = Paragraph::new(input_text)
        .style(Style::default().fg(COLOR_SELECTED).bg(popup_bg));

    // Input at top of inner area (first line)
    let input_area = Rect::new(inner_area.x, inner_area.y, inner_area.width, 1);

    frame.render_widget(input, input_area);
}

/// Get the abbreviated weekday name.
fn weekday_abbr(weekday: chrono::Weekday) -> &'static str {
    match weekday {
        chrono::Weekday::Mon => "Mon",
        chrono::Weekday::Tue => "Tue",
        chrono::Weekday::Wed => "Wed",
        chrono::Weekday::Thu => "Thu",
        chrono::Weekday::Fri => "Fri",
        chrono::Weekday::Sat => "Sat",
        chrono::Weekday::Sun => "Sun",
    }
}

/// Get the number of days in a month.
fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

/// Render the week view with grid, details, and notes panes.
fn render_week_view(model: &PlannerModel, frame: &mut Frame) {
    let area = frame.area();

    // Main layout: top section (grid + details) | bottom section (notes)
    let vertical_chunks = Layout::vertical([
        Constraint::Min(10),      // Grid + Details (takes most space)
        Constraint::Length(10),   // Notes section (taller for vertical list)
    ])
    .split(area);

    // Top section: grid pane (50%) | details pane (50%)
    let main_chunks = Layout::horizontal([
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ])
    .split(vertical_chunks[0]);

    // Get month data for the selected week
    let year = model.selected_date.year();
    let month = model.selected_date.month();
    let month_data = model
        .month_data
        .get(&(year, month))
        .cloned()
        .unwrap_or_else(|| crate::data::sample_data(year, month));

    // Render grid and details panes
    render_week_grid_pane(model, frame, main_chunks[0], &month_data);
    render_week_details_pane(model, frame, main_chunks[1], &month_data);

    // Render notes section
    render_week_notes_pane(model, frame, vertical_chunks[1]);

    // Render popup overlay if in input mode
    match model.input_mode {
        InputMode::Normal => {}
        InputMode::ContextMenu => render_context_menu(model, frame, area),
        InputMode::ConfirmDelete => render_confirm_delete(model, frame, area),
        InputMode::AddingEvent | InputMode::EditingEvent => {
            render_event_form(model, frame, area)
        }
        InputMode::AddingGoal
        | InputMode::AddingIntention
        | InputMode::EditingGoal
        | InputMode::EditingIntention
        | InputMode::AddingWeekNote
        | InputMode::EditingWeekNote => render_popup(model, frame, area),
        InputMode::ViewingWeekNote => render_week_note_view_popup(model, frame, area),
    }
}

/// Render the week grid pane (left side).
fn render_week_grid_pane(model: &PlannerModel, frame: &mut Frame, area: Rect, month_data: &MonthData) {
    let is_focused = model.week_pane == WeekPane::Grid;

    // Layout: content + help line
    let chunks = Layout::vertical([
        Constraint::Min(0),    // Grid content
        Constraint::Length(1), // Help line
    ])
    .split(area);

    // Calculate week start (Monday) from selected date
    let week_start = model.selected_date - chrono::Duration::days(model.selected_date.weekday().num_days_from_monday() as i64);
    let week_end = week_start + chrono::Duration::days(6);

    // Title with week range
    let title = format!(
        " {} {} - {} {} ",
        week_start.format("%b"),
        week_start.day(),
        week_end.format("%b"),
        week_end.day()
    );

    let border_color = if is_focused {
        COLOR_FOCUSED_BORDER
    } else {
        COLOR_UNFOCUSED_BORDER
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    let inner_area = block.inner(chunks[0]);
    frame.render_widget(block, chunks[0]);

    // Render the grid
    render_week_grid(model, frame, inner_area, month_data, week_start, is_focused);

    // Help line
    let help = if is_focused {
        "h/l: day │ j/k: hour │ Tab: details │ Enter: day │ Esc: month"
    } else {
        ""
    };
    let help_paragraph = Paragraph::new(help)
        .style(Style::default().fg(COLOR_HELP))
        .alignment(Alignment::Center);
    frame.render_widget(help_paragraph, chunks[1]);
}

/// Render the week grid with hours as rows and days as columns.
fn render_week_grid(
    model: &PlannerModel,
    frame: &mut Frame,
    area: Rect,
    month_data: &MonthData,
    week_start: NaiveDate,
    is_focused: bool,
) {
    // Get the month/year of our month_data from the model's selected_date
    let data_year = model.selected_date.year();
    let data_month = model.selected_date.month();

    // Column widths: time column (8) + 7 day columns
    let time_col_width: u16 = 9;
    let day_col_width = (area.width.saturating_sub(time_col_width)) / 7;

    // Header row with day names and dates
    let mut header_spans = vec![Span::raw(format!("{:width$}", "", width = time_col_width as usize))];

    for day_offset in 0..7u8 {
        let date = week_start + chrono::Duration::days(day_offset as i64);
        let weekday = date.weekday();
        let is_today = date == model.today;
        let is_selected = is_focused && day_offset == model.week_selected_day;
        let is_weekend = matches!(weekday, chrono::Weekday::Sat | chrono::Weekday::Sun);

        let day_abbr = weekday_abbr(weekday);
        let day_num = date.day();
        let header_text = format!("{:^width$}", format!("{} {}", day_abbr, day_num), width = day_col_width as usize);

        let style = if is_today && is_selected {
            Style::default().fg(COLOR_TODAY).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else if is_today {
            Style::default().fg(COLOR_TODAY).add_modifier(Modifier::BOLD)
        } else if is_selected {
            Style::default().fg(COLOR_SELECTED).add_modifier(Modifier::BOLD)
        } else if is_weekend {
            Style::default().fg(COLOR_WEEKEND)
        } else {
            Style::default().fg(COLOR_WEEKDAY)
        };

        header_spans.push(Span::styled(header_text, style));
    }

    let header_line = Line::from(header_spans);
    frame.render_widget(Paragraph::new(header_line), Rect::new(area.x, area.y, area.width, 1));

    // Separator line
    let separator = "─".repeat(area.width as usize);
    frame.render_widget(
        Paragraph::new(separator).style(Style::default().fg(COLOR_UNFOCUSED_BORDER)),
        Rect::new(area.x, area.y + 1, area.width, 1),
    );

    // Hour rows
    let grid_area = Rect::new(area.x, area.y + 2, area.width, area.height.saturating_sub(2));
    let current_hour = Local::now().hour() as u8;

    // Build event map for the week
    let mut week_events: std::collections::HashMap<(u8, u8), Vec<&crate::data::Event>> =
        std::collections::HashMap::new();

    for day_offset in 0..7u8 {
        let date = week_start + chrono::Duration::days(day_offset as i64);
        // Only get events if this date is in the same month as our month_data
        if date.month() == data_month && date.year() == data_year {
            let events = month_data.events_for_day(date.day());
            for event in events {
                if let Some(start) = event.start_time {
                    let hour = start.hour() as u8;
                    week_events.entry((day_offset, hour)).or_default().push(event);
                }
            }
        }
    }

    let mut lines: Vec<Line> = Vec::with_capacity(24);

    for hour in 0..24u8 {
        let is_current_hour = hour == current_hour;
        let is_selected_hour = is_focused && hour == model.week_selected_hour;

        // Time label with 12-hour format
        let (hour_12, period) = match hour {
            0 => (12, "am"),
            1..=11 => (hour, "am"),
            12 => (12, "pm"),
            13..=23 => (hour - 12, "pm"),
            _ => (hour, ""),
        };

        let time_str = format!("{:02}:00{}", hour_12, period);
        let time_style = if is_current_hour {
            Style::default().fg(COLOR_CURRENT_HOUR).add_modifier(Modifier::BOLD)
        } else if is_selected_hour {
            Style::default().fg(COLOR_SELECTED)
        } else {
            Style::default().fg(COLOR_HELP)
        };

        let mut row_spans = vec![Span::styled(format!("{:>8} ", time_str), time_style)];

        // Day columns
        for day_offset in 0..7u8 {
            let is_selected_cell = is_focused
                && day_offset == model.week_selected_day
                && hour == model.week_selected_hour;

            let events_at_cell = week_events.get(&(day_offset, hour));
            let has_events = events_at_cell.map_or(false, |e| !e.is_empty());

            let cell_content = if has_events {
                let evts = events_at_cell.unwrap();
                let indicator = if evts.len() > 1 {
                    format!("██+{}", evts.len() - 1)
                } else {
                    "████".to_string()
                };
                format!("{:^width$}", indicator, width = day_col_width as usize)
            } else {
                format!("{:^width$}", "░░", width = day_col_width as usize)
            };

            let cell_style = if is_selected_cell {
                Style::default().fg(COLOR_SELECTED).add_modifier(Modifier::BOLD)
            } else if has_events {
                Style::default().fg(COLOR_EVENT_BLOCK)
            } else {
                Style::default().fg(COLOR_HOUR_EMPTY)
            };

            row_spans.push(Span::styled(cell_content, cell_style));
        }

        lines.push(Line::from(row_spans));
    }

    // Scroll to keep selected hour visible
    let visible_height = grid_area.height;
    let scroll_offset = if visible_height >= 24 {
        0
    } else {
        let selected = model.week_selected_hour as u16;
        let ideal_scroll = selected.saturating_sub(visible_height / 2);
        ideal_scroll.min(24u16.saturating_sub(visible_height))
    };

    let paragraph = Paragraph::new(lines).scroll((scroll_offset, 0));
    frame.render_widget(paragraph, grid_area);
}

/// Render the week details pane (right side).
fn render_week_details_pane(model: &PlannerModel, frame: &mut Frame, area: Rect, month_data: &MonthData) {
    let is_focused = model.week_pane == WeekPane::Details;

    // Layout: content + help line
    let chunks = Layout::vertical([
        Constraint::Min(0),    // Details content
        Constraint::Length(1), // Help line
    ])
    .split(area);

    // Calculate selected date from week
    let selected_date = model.week_selected_date();
    let title = format!(" {} ", selected_date.format("%A, %B %e"));

    let border_color = if is_focused {
        COLOR_FOCUSED_BORDER
    } else {
        COLOR_UNFOCUSED_BORDER
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    let inner_area = block.inner(chunks[0]);
    frame.render_widget(block, chunks[0]);

    // Split inner area: 50% events for selected time slot, 30% goals, 20% intentions
    let section_chunks = Layout::vertical([
        Constraint::Percentage(50),
        Constraint::Percentage(30),
        Constraint::Percentage(20),
    ])
    .split(inner_area);

    render_week_events_section(model, frame, section_chunks[0], month_data, selected_date);
    render_week_goals_section(model, frame, section_chunks[1], month_data);
    render_week_intention_section(model, frame, section_chunks[2], month_data);

    // Help line
    let help = if is_focused {
        "j/k: item │ [/]: section │ Space: menu │ Tab: grid"
    } else {
        ""
    };
    let help_paragraph = Paragraph::new(help)
        .style(Style::default().fg(COLOR_HELP))
        .alignment(Alignment::Center);
    frame.render_widget(help_paragraph, chunks[1]);
}

/// Render events section for the week view showing events on the selected day.
fn render_week_events_section(
    model: &PlannerModel,
    frame: &mut Frame,
    area: Rect,
    month_data: &MonthData,
    selected_date: NaiveDate,
) {
    let is_focused = model.week_pane == WeekPane::Details
        && model.details_focus == DetailsFocus::Events;

    // Get the month/year of our month_data from the model's selected_date
    let data_year = model.selected_date.year();
    let data_month = model.selected_date.month();

    let border_color = if is_focused {
        COLOR_SECTION_HEADER
    } else {
        COLOR_UNFOCUSED_BORDER
    };

    let block = Block::default()
        .title(" Events ")
        .title_style(Style::default().fg(COLOR_SECTION_HEADER).add_modifier(Modifier::BOLD))
        .borders(Borders::TOP)
        .border_style(Style::default().fg(border_color));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    // Only show events if selected date is in the month data
    let events = if selected_date.month() == data_month && selected_date.year() == data_year {
        month_data.events_for_day(selected_date.day())
    } else {
        vec![]
    };

    if events.is_empty() {
        let empty = Paragraph::new("  No events")
            .style(Style::default().fg(COLOR_INVALID_DAY));
        frame.render_widget(empty, inner_area);
        return;
    }

    let mut lines: Vec<Line> = Vec::with_capacity(events.len());

    for (idx, event) in events.iter().enumerate() {
        let is_selected = is_focused && idx == model.selected_event_idx;
        let prefix = if is_selected { " > " } else { "   " };

        // Time range or "all-day"
        let time_str = format_event_time(event);

        let title_style = if is_selected {
            Style::default().fg(COLOR_SELECTED).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let line = Line::from(vec![
            Span::raw(prefix),
            Span::styled(format!("{:12} ", time_str), Style::default().fg(COLOR_HELP)),
            Span::styled(event.title.clone(), title_style),
        ]);
        lines.push(line);
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner_area);
}

/// Render goals section for week view (shows month goals).
fn render_week_goals_section(model: &PlannerModel, frame: &mut Frame, area: Rect, month_data: &MonthData) {
    let is_focused = model.week_pane == WeekPane::Details
        && model.details_focus == DetailsFocus::Goals;

    let border_color = if is_focused {
        COLOR_SECTION_HEADER
    } else {
        COLOR_UNFOCUSED_BORDER
    };

    let block = Block::default()
        .title(" Goals (Month) ")
        .title_style(Style::default().fg(COLOR_SECTION_HEADER).add_modifier(Modifier::BOLD))
        .borders(Borders::TOP)
        .border_style(Style::default().fg(border_color));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    if month_data.goals.is_empty() {
        let empty = Paragraph::new("  No goals")
            .style(Style::default().fg(COLOR_INVALID_DAY));
        frame.render_widget(empty, inner_area);
        return;
    }

    let mut lines: Vec<Line> = Vec::with_capacity(month_data.goals.len());

    for (idx, goal) in month_data.goals.iter().enumerate() {
        let is_selected = is_focused && idx == model.selected_goal_idx;
        let prefix = if is_selected { " > " } else { "   " };

        let checkbox = if goal.completed { "[x]" } else { "[ ]" };
        let checkbox_color = if goal.completed {
            COLOR_GOAL_COMPLETED
        } else {
            COLOR_GOAL_PENDING
        };

        let title_style = if is_selected {
            Style::default().fg(COLOR_SELECTED).add_modifier(Modifier::BOLD)
        } else if goal.completed {
            Style::default().fg(COLOR_HELP)
        } else {
            Style::default()
        };

        let line = Line::from(vec![
            Span::raw(prefix),
            Span::styled(checkbox, Style::default().fg(checkbox_color)),
            Span::raw(" "),
            Span::styled(goal.title.clone(), title_style),
        ]);
        lines.push(line);
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner_area);
}

/// Render the week notes pane at the bottom of week view.
fn render_week_notes_pane(model: &PlannerModel, frame: &mut Frame, area: Rect) {
    let is_focused = model.week_pane == WeekPane::Notes;

    // Calculate week start for title
    let week_start = model.selected_date
        - chrono::Duration::days(model.selected_date.weekday().num_days_from_monday() as i64);
    let week_end = week_start + chrono::Duration::days(6);
    let title = format!(
        " Week Notes ({} - {}) ",
        week_start.format("%b %d"),
        week_end.format("%b %d")
    );

    let border_color = if is_focused {
        COLOR_FOCUSED_BORDER
    } else {
        COLOR_UNFOCUSED_BORDER
    };

    // Help text at bottom - different when focused
    let help_text = if is_focused {
        " j/k: navigate │ n: new │ e: edit │ d: delete │ Enter: view │ Tab: grid "
    } else {
        " Tab to focus │ n: new "
    };

    let block = Block::default()
        .title(title)
        .title_style(Style::default().fg(COLOR_SECTION_HEADER).add_modifier(Modifier::BOLD))
        .title_bottom(Line::from(help_text).right_aligned())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    // Get notes from cache (use immutable access to avoid borrow issues)
    let monday = model.current_week_monday();
    let notes = model.week_notes.get(&monday);

    let empty_vec = Vec::new();
    let notes = notes.unwrap_or(&empty_vec);

    if notes.is_empty() {
        let empty = Paragraph::new("  No notes for this week. Press 'n' to add one.")
            .style(Style::default().fg(COLOR_INVALID_DAY));
        frame.render_widget(empty, inner_area);
        return;
    }

    // Render notes as a vertical bullet list with scrolling
    let mut lines: Vec<Line> = Vec::with_capacity(notes.len());

    for (idx, note) in notes.iter().enumerate() {
        let is_selected = is_focused && idx == model.selected_week_note_idx;
        let prefix = if is_selected { " > " } else { "   " };

        // Bullet style
        let bullet_style = if is_selected {
            Style::default().fg(COLOR_SELECTED)
        } else {
            Style::default().fg(COLOR_EVENT_DOT)
        };

        // Text style
        let text_style = if is_selected {
            Style::default().fg(COLOR_SELECTED).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        // Calculate available width for note text (leave room for prefix + bullet + padding)
        let available_width = inner_area.width.saturating_sub(8) as usize;

        // Truncate long notes for display, but show more than before
        let display_text = if note.text.len() > available_width {
            format!("{}...", &note.text[..available_width.saturating_sub(3)])
        } else {
            note.text.clone()
        };

        let line = Line::from(vec![
            Span::raw(prefix),
            Span::styled("• ", bullet_style),
            Span::styled(display_text, text_style),
        ]);
        lines.push(line);
    }

    // Apply scroll offset
    let scroll_offset = model.week_notes_scroll as u16;
    let paragraph = Paragraph::new(lines).scroll((scroll_offset, 0));
    frame.render_widget(paragraph, inner_area);
}

/// Render popup for viewing the full week note.
fn render_week_note_view_popup(model: &PlannerModel, frame: &mut Frame, area: Rect) {
    // Subtle dim effect
    let dim_style = Style::default().fg(Color::DarkGray).bg(Color::Rgb(28, 31, 38));
    let dim_block = Block::default().style(dim_style);
    frame.render_widget(dim_block, area);

    // Popup dimensions - 60% width, up to 80% height
    let popup_width = (area.width * 60 / 100).max(50).min(area.width.saturating_sub(4));
    let popup_height = (area.height * 80 / 100).max(10).min(area.height.saturating_sub(4));

    // Center over the full screen area
    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // Clear the area behind the popup
    frame.render_widget(Clear, popup_area);

    // Popup background color
    let popup_bg = Color::Rgb(40, 44, 52);

    // Help text
    let help_text = " Esc: close │ e: edit │ d: delete ";

    let block = Block::default()
        .title(" View Note ")
        .title_style(Style::default().fg(COLOR_SECTION_HEADER).add_modifier(Modifier::BOLD))
        .title_bottom(Line::from(help_text).centered())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(COLOR_FOCUSED_BORDER))
        .style(Style::default().bg(popup_bg));

    let inner_area = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    // Get the selected note
    let monday = model.current_week_monday();
    if let Some(notes) = model.week_notes.get(&monday) {
        if let Some(note) = notes.get(model.selected_week_note_idx) {
            // Render the note text with word wrapping
            let paragraph = Paragraph::new(note.text.as_str())
                .style(Style::default().fg(Color::White).bg(popup_bg))
                .wrap(Wrap { trim: false });
            frame.render_widget(paragraph, inner_area);
        }
    }
}

/// Render intentions section for week view (shows month intentions).
fn render_week_intention_section(model: &PlannerModel, frame: &mut Frame, area: Rect, month_data: &MonthData) {
    let is_focused = model.week_pane == WeekPane::Details
        && model.details_focus == DetailsFocus::Intention;

    let border_color = if is_focused {
        COLOR_SECTION_HEADER
    } else {
        COLOR_UNFOCUSED_BORDER
    };

    let block = Block::default()
        .title(" Intentions ")
        .title_style(Style::default().fg(COLOR_SECTION_HEADER).add_modifier(Modifier::BOLD))
        .borders(Borders::TOP)
        .border_style(Style::default().fg(border_color));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    if month_data.intentions.is_empty() {
        let empty = Paragraph::new("  No intentions set")
            .style(Style::default().fg(COLOR_INVALID_DAY));
        frame.render_widget(empty, inner_area);
        return;
    }

    let mut lines: Vec<Line> = Vec::with_capacity(month_data.intentions.len());

    for (idx, intention) in month_data.intentions.iter().enumerate() {
        let is_selected = is_focused && idx == model.selected_intention_idx;
        let prefix = if is_selected { " > " } else { "   " };

        let text_style = if is_selected {
            Style::default().fg(COLOR_SELECTED).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let line = Line::from(vec![
            Span::raw(prefix),
            Span::styled("• ", Style::default().fg(COLOR_EVENT_DOT)),
            Span::styled(intention.text.clone(), text_style),
        ]);
        lines.push(line);
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner_area);
}

/// Render the day view with timeline and details panes.
fn render_day_view(model: &PlannerModel, frame: &mut Frame) {
    let area = frame.area();

    // Main layout: timeline pane (40%) | details pane (60%)
    let main_chunks = Layout::horizontal([
        Constraint::Percentage(40),
        Constraint::Percentage(60),
    ])
    .split(area);

    // Get month data for the selected date
    let year = model.selected_date.year();
    let month = model.selected_date.month();
    let month_data = model
        .month_data
        .get(&(year, month))
        .cloned()
        .unwrap_or_else(|| crate::data::sample_data(year, month));

    // Render both panes
    render_timeline_pane(model, frame, main_chunks[0], &month_data);
    render_day_details_pane(model, frame, main_chunks[1], &month_data);

    // Render popup overlay if in input mode
    match model.input_mode {
        InputMode::Normal => {}
        InputMode::ContextMenu => render_context_menu(model, frame, area),
        InputMode::ConfirmDelete => render_confirm_delete(model, frame, area),
        InputMode::AddingEvent | InputMode::EditingEvent => {
            render_event_form(model, frame, area)
        }
        InputMode::AddingGoal
        | InputMode::AddingIntention
        | InputMode::EditingGoal
        | InputMode::EditingIntention
        | InputMode::AddingWeekNote
        | InputMode::EditingWeekNote => render_popup(model, frame, area),
        InputMode::ViewingWeekNote => {} // Not used in day view
    }
}

/// Render the timeline pane (left side of day view).
fn render_timeline_pane(model: &PlannerModel, frame: &mut Frame, area: Rect, month_data: &MonthData) {
    let is_focused = model.day_pane == DayPane::Timeline;

    // Layout: content + help line
    let chunks = Layout::vertical([
        Constraint::Min(0),    // Timeline content
        Constraint::Length(1), // Help line
    ])
    .split(area);

    // Title with full date
    let title = format!(" {} ", model.selected_date.format("%A, %B %e, %Y"));

    let border_color = if is_focused {
        COLOR_FOCUSED_BORDER
    } else {
        COLOR_UNFOCUSED_BORDER
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    let inner_area = block.inner(chunks[0]);
    frame.render_widget(block, chunks[0]);

    // Get events for this day
    let day = model.selected_date.day();
    let events = month_data.events_for_day(day);
    let current_hour = Local::now().hour() as u8;

    // Separate all-day events
    let all_day_events: Vec<_> = events.iter().filter(|e| e.start_time.is_none()).copied().collect();
    let timed_events: Vec<_> = events.iter().filter(|e| e.start_time.is_some()).copied().collect();

    // Layout within timeline: all-day section + hour grid
    let has_all_day = !all_day_events.is_empty();
    let all_day_height = if has_all_day { (all_day_events.len() as u16).min(3) + 1 } else { 0 };

    let timeline_chunks = Layout::vertical([
        Constraint::Length(all_day_height),
        Constraint::Min(0), // Hour grid
    ])
    .split(inner_area);

    // Render all-day events section
    if has_all_day {
        render_all_day_section(frame, timeline_chunks[0], &all_day_events, is_focused);
    }

    // Render hour grid
    render_hour_grid(
        model,
        frame,
        timeline_chunks[1],
        &timed_events,
        current_hour,
        is_focused,
    );

    // Help line
    let help = if is_focused {
        "j/k: hour │ h/l: day │ g: now │ Tab: details │ Esc: month"
    } else {
        ""
    };
    let help_paragraph = Paragraph::new(help)
        .style(Style::default().fg(COLOR_HELP))
        .alignment(Alignment::Center);
    frame.render_widget(help_paragraph, chunks[1]);
}

/// Render the all-day events section at the top of the timeline.
fn render_all_day_section(
    frame: &mut Frame,
    area: Rect,
    events: &[&crate::data::Event],
    _is_focused: bool,
) {
    let mut lines: Vec<Line> = Vec::with_capacity(events.len() + 1);

    for event in events.iter().take(3) {
        let line = Line::from(vec![
            Span::styled("▌", Style::default().fg(COLOR_EVENT_BLOCK)),
            Span::styled("all-day", Style::default().fg(COLOR_HELP)),
            Span::styled("▐ ", Style::default().fg(COLOR_EVENT_BLOCK)),
            Span::styled(event.title.as_str(), Style::default().fg(Color::White)),
        ]);
        lines.push(line);
    }

    // Separator line
    if !events.is_empty() {
        let separator = "─".repeat(area.width.saturating_sub(1) as usize);
        lines.push(Line::from(Span::styled(separator, Style::default().fg(COLOR_UNFOCUSED_BORDER))));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

/// Render the hour grid with events.
fn render_hour_grid(
    model: &PlannerModel,
    frame: &mut Frame,
    area: Rect,
    events: &[&crate::data::Event],
    current_hour: u8,
    is_focused: bool,
) {
    // Build a map of hour -> events
    let mut hour_events: std::collections::HashMap<u8, Vec<&crate::data::Event>> =
        std::collections::HashMap::new();
    for event in events {
        if let Some(start) = event.start_time {
            let hour = start.hour() as u8;
            hour_events.entry(hour).or_default().push(*event);
        }
    }

    // Always render all 24 hours
    let mut lines: Vec<Line> = Vec::with_capacity(24);

    for hour in 0..24u8 {
        let is_current = hour == current_hour && model.selected_date == model.today;
        let is_selected = is_focused && hour == model.selected_hour;
        let events_at_hour = hour_events.get(&hour);

        let line = format_hour_row(hour, is_current, is_selected, events_at_hour, area.width);
        lines.push(line);
    }

    // Scroll to keep selected hour visible
    let visible_height = area.height as u16;
    let scroll_offset = if visible_height >= 24 {
        0 // All hours fit, no scroll needed
    } else {
        let selected = model.selected_hour as u16;
        // Keep selected hour roughly centered, but clamp to valid range
        let ideal_scroll = selected.saturating_sub(visible_height / 2);
        ideal_scroll.min(24u16.saturating_sub(visible_height))
    };

    let paragraph = Paragraph::new(lines).scroll((scroll_offset, 0));
    frame.render_widget(paragraph, area);
}

/// Format a single hour row in the timeline.
fn format_hour_row(
    hour: u8,
    is_current: bool,
    is_selected: bool,
    events: Option<&Vec<&crate::data::Event>>,
    width: u16,
) -> Line<'static> {
    // Convert to 12-hour format with minutes and leading zeros
    let (hour_12, period) = match hour {
        0 => (12, "am"),
        1..=11 => (hour, "am"),
        12 => (12, "pm"),
        13..=23 => (hour - 12, "pm"),
        _ => (hour, ""),
    };
    let hour_str = format!("{:02}:00{}", hour_12, period);

    // Hour label style
    let hour_style = if is_current {
        Style::default().fg(COLOR_CURRENT_HOUR).add_modifier(Modifier::BOLD)
    } else if is_selected {
        Style::default().fg(COLOR_SELECTED).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(COLOR_HELP)
    };

    // Selection indicator
    let prefix = if is_selected { "▶ " } else { "  " };
    let prefix_style = if is_selected {
        Style::default().fg(COLOR_SELECTED)
    } else {
        Style::default()
    };

    // Build the line - content fills remaining width
    // prefix (2) + hour (7, e.g. "12:00pm") + space (1) = 10 chars before content
    // Reserve 2 chars at end for potential " ◀" indicator
    let content_width = width.saturating_sub(12) as usize;

    let content = match events {
        Some(evts) if !evts.is_empty() => {
            let first_event = &evts[0];
            let title = &first_event.title;
            let extra = if evts.len() > 1 {
                format!(" +{}", evts.len() - 1)
            } else {
                String::new()
            };
            // Pad to full width with spaces after the event info
            let display = format!("█ {}{}", title, extra);
            if display.len() > content_width {
                format!("{:width$}", &display[..content_width.saturating_sub(3)], width = content_width)
            } else {
                format!("{:width$}", display, width = content_width)
            }
        }
        _ => {
            // Empty hour - fill entire width with dim block
            "░".repeat(content_width)
        }
    };

    let content_style = match events {
        Some(evts) if !evts.is_empty() => {
            if is_selected {
                Style::default().fg(COLOR_SELECTED)
            } else {
                Style::default().fg(COLOR_EVENT_BLOCK)
            }
        }
        _ => Style::default().fg(COLOR_HOUR_EMPTY),
    };

    // Current hour indicator
    let now_indicator = if is_current { " ◀" } else { "" };

    Line::from(vec![
        Span::styled(prefix, prefix_style),
        Span::styled(hour_str, hour_style),
        Span::raw(" "),
        Span::styled(content, content_style),
        Span::styled(now_indicator, Style::default().fg(COLOR_CURRENT_HOUR)),
    ])
}

/// Render the day details pane (right side of day view).
fn render_day_details_pane(model: &PlannerModel, frame: &mut Frame, area: Rect, month_data: &MonthData) {
    let is_focused = model.day_pane == DayPane::Details;

    // Layout: content + help line
    let chunks = Layout::vertical([
        Constraint::Min(0),    // Details content
        Constraint::Length(1), // Help line
    ])
    .split(area);

    // Title with "Details"
    let title = " Details ";

    let border_color = if is_focused {
        COLOR_FOCUSED_BORDER
    } else {
        COLOR_UNFOCUSED_BORDER
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    let inner_area = block.inner(chunks[0]);
    frame.render_widget(block, chunks[0]);

    // Split inner area into sections: 50% events, 30% goals, 20% intention
    let section_chunks = Layout::vertical([
        Constraint::Percentage(50),
        Constraint::Percentage(30),
        Constraint::Percentage(20),
    ])
    .split(inner_area);

    // Reuse the month view section renderers with day pane focus
    render_day_events_section(model, frame, section_chunks[0], month_data);
    render_day_goals_section(model, frame, section_chunks[1], month_data);
    render_day_intention_section(model, frame, section_chunks[2], month_data);

    // Help line
    let help = if is_focused {
        "j/k: item │ [/]: section │ Space: menu │ Tab: timeline"
    } else {
        ""
    };
    let help_paragraph = Paragraph::new(help)
        .style(Style::default().fg(COLOR_HELP))
        .alignment(Alignment::Center);
    frame.render_widget(help_paragraph, chunks[1]);
}

/// Render the events section in day view details pane.
fn render_day_events_section(model: &PlannerModel, frame: &mut Frame, area: Rect, month_data: &MonthData) {
    let is_focused = model.day_pane == DayPane::Details
        && model.details_focus == DetailsFocus::Events;

    let border_color = if is_focused {
        COLOR_SECTION_HEADER
    } else {
        COLOR_UNFOCUSED_BORDER
    };

    let block = Block::default()
        .title(" Events ")
        .title_style(Style::default().fg(COLOR_SECTION_HEADER).add_modifier(Modifier::BOLD))
        .borders(Borders::TOP)
        .border_style(Style::default().fg(border_color));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let day = model.selected_date.day();
    let events = month_data.events_for_day(day);

    if events.is_empty() {
        let empty = Paragraph::new("  No events")
            .style(Style::default().fg(COLOR_INVALID_DAY));
        frame.render_widget(empty, inner_area);
        return;
    }

    let mut lines: Vec<Line> = Vec::with_capacity(events.len());

    for (idx, event) in events.iter().enumerate() {
        let is_selected = is_focused && idx == model.selected_event_idx;
        let prefix = if is_selected { " > " } else { "   " };

        // Time range or "all-day"
        let time_str = format_event_time(event);

        let title_style = if is_selected {
            Style::default().fg(COLOR_SELECTED).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let line = Line::from(vec![
            Span::raw(prefix),
            Span::styled(format!("{:12} ", time_str), Style::default().fg(COLOR_HELP)),
            Span::styled(event.title.clone(), title_style),
        ]);
        lines.push(line);
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner_area);
}

/// Render the goals section in day view details pane.
fn render_day_goals_section(model: &PlannerModel, frame: &mut Frame, area: Rect, month_data: &MonthData) {
    let is_focused = model.day_pane == DayPane::Details
        && model.details_focus == DetailsFocus::Goals;

    let border_color = if is_focused {
        COLOR_SECTION_HEADER
    } else {
        COLOR_UNFOCUSED_BORDER
    };

    let block = Block::default()
        .title(" Goals (Month) ")
        .title_style(Style::default().fg(COLOR_SECTION_HEADER).add_modifier(Modifier::BOLD))
        .borders(Borders::TOP)
        .border_style(Style::default().fg(border_color));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    if month_data.goals.is_empty() {
        let empty = Paragraph::new("  No goals")
            .style(Style::default().fg(COLOR_INVALID_DAY));
        frame.render_widget(empty, inner_area);
        return;
    }

    let mut lines: Vec<Line> = Vec::with_capacity(month_data.goals.len());

    for (idx, goal) in month_data.goals.iter().enumerate() {
        let is_selected = is_focused && idx == model.selected_goal_idx;
        let prefix = if is_selected { " > " } else { "   " };

        let checkbox = if goal.completed { "[x]" } else { "[ ]" };
        let checkbox_color = if goal.completed {
            COLOR_GOAL_COMPLETED
        } else {
            COLOR_GOAL_PENDING
        };

        let title_style = if is_selected {
            Style::default().fg(COLOR_SELECTED).add_modifier(Modifier::BOLD)
        } else if goal.completed {
            Style::default().fg(COLOR_HELP)
        } else {
            Style::default()
        };

        let line = Line::from(vec![
            Span::raw(prefix),
            Span::styled(checkbox, Style::default().fg(checkbox_color)),
            Span::raw(" "),
            Span::styled(goal.title.clone(), title_style),
        ]);
        lines.push(line);
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner_area);
}

/// Render the intentions section in day view details pane.
fn render_day_intention_section(model: &PlannerModel, frame: &mut Frame, area: Rect, month_data: &MonthData) {
    let is_focused = model.day_pane == DayPane::Details
        && model.details_focus == DetailsFocus::Intention;

    let border_color = if is_focused {
        COLOR_SECTION_HEADER
    } else {
        COLOR_UNFOCUSED_BORDER
    };

    let block = Block::default()
        .title(" Intentions ")
        .title_style(Style::default().fg(COLOR_SECTION_HEADER).add_modifier(Modifier::BOLD))
        .borders(Borders::TOP)
        .border_style(Style::default().fg(border_color));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    if month_data.intentions.is_empty() {
        let empty = Paragraph::new("  No intentions set")
            .style(Style::default().fg(COLOR_INVALID_DAY));
        frame.render_widget(empty, inner_area);
        return;
    }

    let mut lines: Vec<Line> = Vec::with_capacity(month_data.intentions.len());

    for (idx, intention) in month_data.intentions.iter().enumerate() {
        let is_selected = is_focused && idx == model.selected_intention_idx;
        let prefix = if is_selected { " > " } else { "   " };

        let text_style = if is_selected {
            Style::default().fg(COLOR_SELECTED).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let line = Line::from(vec![
            Span::raw(prefix),
            Span::styled("• ", Style::default().fg(COLOR_EVENT_DOT)),
            Span::styled(intention.text.clone(), text_style),
        ]);
        lines.push(line);
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner_area);
}

/// Get the name of a month.
fn month_name(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_year_to_words() {
        assert_eq!(year_to_words(2026), "TWENTY TWENTY SIX");
        assert_eq!(year_to_words(2025), "TWENTY TWENTY FIVE");
        assert_eq!(year_to_words(2000), "TWENTY HUNDRED");
        assert_eq!(year_to_words(1999), "NINETEEN NINETY NINE");
        assert_eq!(year_to_words(2010), "TWENTY TEN");
    }
}
