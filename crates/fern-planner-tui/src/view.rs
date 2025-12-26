//! View rendering for the planner TUI.

use chrono::{Datelike, NaiveDate};
use frond::prelude::*;

use crate::data::MonthData;
use crate::model::{DetailsFocus, EventFormField, InputMode, MenuAction, MonthPane, PlannerModel, View};

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
        | InputMode::EditingIntention => render_popup(model, frame, area),
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

        // Time or "all-day"
        let time_str = match event.time {
            Some(t) => t.format("%H:%M").to_string(),
            None => "all-day".to_string(),
        };

        let title_style = if is_selected {
            Style::default().fg(COLOR_SELECTED).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let line = Line::from(vec![
            Span::raw(prefix),
            Span::styled(format!("{:7} ", time_str), Style::default().fg(COLOR_HELP)),
            Span::styled(&event.title, title_style),
        ]);
        lines.push(line);

        // Description on next line if selected
        if is_selected {
            if let Some(ref desc) = event.description {
                let desc_line = Line::from(vec![
                    Span::raw("          "),
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

/// Render the context menu popup for add/edit/delete actions.
fn render_context_menu(model: &PlannerModel, frame: &mut Frame, area: Rect) {
    // Subtle dim effect
    let dim_style = Style::default().fg(Color::DarkGray).bg(Color::Rgb(28, 31, 38));
    let dim_block = Block::default().style(dim_style);
    frame.render_widget(dim_block, area);

    // Small popup - just 3 menu items
    let popup_width: u16 = 24;
    let popup_height: u16 = 7;

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
    let title = format!(" {} ", section_name);

    let block = Block::default()
        .title(title)
        .title_style(Style::default().fg(COLOR_SECTION_HEADER).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(COLOR_FOCUSED_BORDER))
        .style(Style::default().bg(popup_bg));

    let inner_area = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    // Check if there's a selected item (for enabling edit/delete)
    let has_item = model.has_selected_item();

    // Menu options
    let options = [
        (MenuAction::Add, "Add", true),
        (MenuAction::Edit, "Edit", has_item),
        (MenuAction::Delete, "Delete", has_item),
    ];

    // Render each option
    for (i, (action, label, enabled)) in options.iter().enumerate() {
        let is_selected = model.menu_action == *action;
        let y = inner_area.y + i as u16;

        let style = if !enabled {
            Style::default().fg(Color::DarkGray)
        } else if is_selected {
            Style::default().fg(COLOR_SELECTED)
        } else {
            Style::default().fg(Color::White)
        };

        let prefix = if is_selected { "› " } else { "  " };
        let text = format!("{}{}", prefix, label);
        let para = Paragraph::new(text).style(style);
        frame.render_widget(para, Rect::new(inner_area.x, y, inner_area.width, 1));
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

    // Popup dimensions - 50% width, 12 lines tall (extra line for top margin)
    let popup_width = (area.width * 50 / 100).max(50).min(area.width.saturating_sub(4));
    let popup_height: u16 = 12;

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

    // === Row 2: Time field ===
    let time_y = title_y + 4;
    let is_time_active = model.event_form_field == EventFormField::Time;
    let time_disabled = model.event_all_day;

    let time_label_style = if time_disabled {
        inactive_style
    } else if is_time_active {
        active_style
    } else {
        label_style
    };
    let time_label = Paragraph::new("    Time:").style(time_label_style);
    frame.render_widget(time_label, Rect::new(field_x, time_y, label_width, 1));

    let time_value = if time_disabled {
        "  (all day)".to_string()
    } else {
        let masked = format_time_mask(&model.event_time, is_time_active);
        if is_time_active {
            format!("› {}", masked)
        } else {
            format!("  {}", masked)
        }
    };
    let time_style = if time_disabled {
        placeholder_style
    } else if is_time_active {
        active_style
    } else {
        inactive_style
    };
    let time_para = Paragraph::new(time_value).style(time_style);
    frame.render_widget(time_para, Rect::new(value_x, time_y, value_width, 1));

    // === Row 3: Description field ===
    let desc_y = title_y + 6;
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

    // Title based on input mode (only Goal/Intention use this popup)
    let title = match model.input_mode {
        InputMode::AddingGoal => " Add Goal ",
        InputMode::EditingGoal => " Edit Goal ",
        InputMode::AddingIntention => " Add Intention ",
        InputMode::EditingIntention => " Edit Intention ",
        InputMode::Normal | InputMode::ContextMenu | InputMode::ConfirmDelete | InputMode::AddingEvent | InputMode::EditingEvent => "",
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

/// Render the week view (placeholder).
fn render_week_view(model: &PlannerModel, frame: &mut Frame) {
    let area = frame.area();

    let block = Block::default()
        .title(format!("Week of {}", model.selected_date))
        .borders(Borders::ALL);

    let content = Paragraph::new("Week view - coming soon")
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(content, area);
}

/// Render the day view (placeholder).
fn render_day_view(model: &PlannerModel, frame: &mut Frame) {
    let area = frame.area();

    let block = Block::default()
        .title(format!("{}", model.selected_date.format("%A, %B %e, %Y")))
        .borders(Borders::ALL);

    let content = Paragraph::new("Day view - coming soon")
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(content, area);
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
