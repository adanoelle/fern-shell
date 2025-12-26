//! Application model - the single source of truth for state.

use std::collections::HashMap;

use chrono::{Datelike, Local, NaiveDate, Timelike};

use crate::data::MonthData;

/// The current view mode of the planner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// Year view - months as columns, days as rows
    Year,
    /// Month view - traditional calendar grid
    Month,
    /// Week view - days as columns, hours as rows
    Week,
    /// Day view - hourly schedule
    Day,
}

impl Default for View {
    fn default() -> Self {
        Self::Year
    }
}

/// Which pane is focused in month view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MonthPane {
    /// The days list (left pane)
    #[default]
    Days,
    /// The details pane (right pane)
    Details,
}

/// Which section is focused in the details pane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DetailsFocus {
    /// Events section
    #[default]
    Events,
    /// Goals section
    Goals,
    /// Intention section
    Intention,
}

impl DetailsFocus {
    /// Cycle to the next section.
    pub fn next(self) -> Self {
        match self {
            Self::Events => Self::Goals,
            Self::Goals => Self::Intention,
            Self::Intention => Self::Events,
        }
    }

    /// Cycle to the previous section.
    pub fn prev(self) -> Self {
        match self {
            Self::Events => Self::Intention,
            Self::Goals => Self::Events,
            Self::Intention => Self::Goals,
        }
    }
}

/// Current input mode for popup dialogs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    /// Normal navigation mode
    #[default]
    Normal,
    /// Context menu for add/edit/delete
    ContextMenu,
    /// Confirm delete dialog
    ConfirmDelete,
    /// Adding a new goal
    AddingGoal,
    /// Adding a new intention
    AddingIntention,
    /// Adding a new event (multi-field form)
    AddingEvent,
    /// Editing an existing goal
    EditingGoal,
    /// Editing an existing intention
    EditingIntention,
    /// Editing an existing event
    EditingEvent,
}

/// Menu action options in the context menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MenuAction {
    #[default]
    Add,
    Edit,
    Delete,
}

impl MenuAction {
    /// Get the next menu action.
    pub fn next(self) -> Self {
        match self {
            Self::Add => Self::Edit,
            Self::Edit => Self::Delete,
            Self::Delete => Self::Add,
        }
    }

    /// Get the previous menu action.
    pub fn prev(self) -> Self {
        match self {
            Self::Add => Self::Delete,
            Self::Edit => Self::Add,
            Self::Delete => Self::Edit,
        }
    }
}

/// Which field is focused in the event form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EventFormField {
    #[default]
    Title,
    AllDay,
    Time,
    Description,
}

/// The complete application state.
#[derive(Debug, Clone)]
pub struct PlannerModel {
    // === Navigation ===
    /// Currently selected date
    pub selected_date: NaiveDate,
    /// Today's date (updated on tick)
    pub today: NaiveDate,
    /// Current view mode
    pub view: View,
    /// Year being viewed (for year view scrolling)
    pub view_year: i32,

    // === UI State ===
    /// Whether the app should quit
    pub should_quit: bool,
    /// Kakoune-style count prefix (e.g., "5j" moves 5 days)
    pub count: Option<u32>,

    // === Month View State ===
    /// Which pane is focused in month view
    pub month_pane: MonthPane,
    /// Which section is focused in the details pane
    pub details_focus: DetailsFocus,
    /// Selected event index in the current day
    pub selected_event_idx: usize,
    /// Selected goal index
    pub selected_goal_idx: usize,
    /// Selected intention index
    pub selected_intention_idx: usize,
    /// Cached month data by (year, month)
    pub month_data: HashMap<(i32, u32), MonthData>,

    // === Input State ===
    /// Current input mode (normal or popup)
    pub input_mode: InputMode,
    /// Text buffer for input dialogs
    pub input_buffer: String,
    /// Current menu action selection
    pub menu_action: MenuAction,
    /// ID of item being edited (for edit/delete operations)
    pub editing_item_id: Option<u64>,

    // === Event Form State ===
    /// Current field in event form
    pub event_form_field: EventFormField,
    /// Event title input
    pub event_title: String,
    /// Event all-day toggle
    pub event_all_day: bool,
    /// Event time input (HH:MM format)
    pub event_time: String,
    /// Event description input
    pub event_description: String,
}

impl Default for PlannerModel {
    fn default() -> Self {
        let today = Local::now().date_naive();
        Self {
            selected_date: today,
            today,
            view: View::Year,
            view_year: today.year(),
            should_quit: false,
            count: None,
            month_pane: MonthPane::default(),
            details_focus: DetailsFocus::default(),
            selected_event_idx: 0,
            selected_goal_idx: 0,
            selected_intention_idx: 0,
            month_data: HashMap::new(),
            input_mode: InputMode::default(),
            input_buffer: String::new(),
            menu_action: MenuAction::default(),
            editing_item_id: None,
            // Event form defaults
            event_form_field: EventFormField::default(),
            event_title: String::new(),
            event_all_day: false,
            event_time: String::new(),
            event_description: String::new(),
        }
    }
}

impl PlannerModel {
    /// Create a new model with default values.
    pub fn new() -> Self {
        Self::default()
    }

    // === Builder-style setters for immutable updates ===

    /// Set the selected date.
    #[must_use]
    pub fn with_selected_date(mut self, date: NaiveDate) -> Self {
        self.selected_date = date;
        self
    }

    /// Set today's date.
    #[must_use]
    pub fn with_today(mut self, date: NaiveDate) -> Self {
        self.today = date;
        self
    }

    /// Set the view mode.
    #[must_use]
    pub fn with_view(mut self, view: View) -> Self {
        self.view = view;
        self
    }

    /// Set the view year.
    #[must_use]
    pub fn with_view_year(mut self, year: i32) -> Self {
        self.view_year = year;
        self
    }

    /// Mark the app to quit.
    #[must_use]
    pub fn with_should_quit(mut self, quit: bool) -> Self {
        self.should_quit = quit;
        self
    }

    /// Clear the count prefix.
    #[must_use]
    pub fn clear_count(mut self) -> Self {
        self.count = None;
        self
    }

    /// Accumulate a digit into the count (Kakoune-style).
    /// Overflow-protected with a maximum count of 999.
    #[must_use]
    pub fn push_digit(mut self, digit: u32) -> Self {
        const MAX_COUNT: u32 = 999;
        let new_count = self
            .count
            .unwrap_or(0)
            .saturating_mul(10)
            .saturating_add(digit)
            .min(MAX_COUNT);
        self.count = Some(new_count);
        self
    }

    /// Get the effective count (defaults to 1 if not set).
    pub fn effective_count(&self) -> u32 {
        self.count.unwrap_or(1)
    }

    // === Navigation helpers ===

    /// Get the first day of the view year.
    fn year_start(&self) -> NaiveDate {
        NaiveDate::from_ymd_opt(self.view_year, 1, 1).unwrap()
    }

    /// Get the last day of the view year.
    fn year_end(&self) -> NaiveDate {
        NaiveDate::from_ymd_opt(self.view_year, 12, 31).unwrap()
    }

    /// Move selection forward by n days, clamped to year end.
    #[must_use]
    pub fn next_day_n(self, n: u32) -> Self {
        let next = self.selected_date + chrono::Days::new(n as u64);
        let clamped = next.min(self.year_end());
        self.with_selected_date(clamped)
    }

    /// Move selection backward by n days, clamped to year start.
    #[must_use]
    pub fn prev_day_n(self, n: u32) -> Self {
        let prev = self.selected_date - chrono::Days::new(n as u64);
        let clamped = prev.max(self.year_start());
        self.with_selected_date(clamped)
    }

    /// Move selection to the next day.
    #[must_use]
    pub fn next_day(self) -> Self {
        self.next_day_n(1)
    }

    /// Move selection to the previous day.
    #[must_use]
    pub fn prev_day(self) -> Self {
        self.prev_day_n(1)
    }

    /// Move selection forward by n weeks, clamped to year end.
    #[must_use]
    pub fn next_week_n(self, n: u32) -> Self {
        let next = self.selected_date + chrono::Days::new(7 * n as u64);
        let clamped = next.min(self.year_end());
        self.with_selected_date(clamped)
    }

    /// Move selection backward by n weeks, clamped to year start.
    #[must_use]
    pub fn prev_week_n(self, n: u32) -> Self {
        let prev = self.selected_date - chrono::Days::new(7 * n as u64);
        let clamped = prev.max(self.year_start());
        self.with_selected_date(clamped)
    }

    /// Move selection to the next week.
    #[must_use]
    pub fn next_week(self) -> Self {
        self.next_week_n(1)
    }

    /// Move selection to the previous week.
    #[must_use]
    pub fn prev_week(self) -> Self {
        self.prev_week_n(1)
    }

    /// Move selection forward by n months, clamped to year end.
    #[must_use]
    pub fn next_month_n(mut self, n: u32) -> Self {
        let year_end = self.year_end();
        for _ in 0..n {
            let current = self.selected_date;
            // Stop if we're already at year end
            if current >= year_end {
                break;
            }
            let next = if current.month() == 12 {
                // Don't go past December of view year
                break;
            } else {
                NaiveDate::from_ymd_opt(current.year(), current.month() + 1, 1)
            };

            if let Some(date) = next {
                let day = current.day().min(days_in_month(date.year(), date.month()));
                if let Some(final_date) = NaiveDate::from_ymd_opt(date.year(), date.month(), day) {
                    self.selected_date = final_date.min(year_end);
                }
            }
        }
        self
    }

    /// Move selection backward by n months, clamped to year start.
    #[must_use]
    pub fn prev_month_n(mut self, n: u32) -> Self {
        let year_start = self.year_start();
        for _ in 0..n {
            let current = self.selected_date;
            // Stop if we're already at year start
            if current <= year_start {
                break;
            }
            let prev = if current.month() == 1 {
                // Don't go before January of view year
                break;
            } else {
                NaiveDate::from_ymd_opt(current.year(), current.month() - 1, 1)
            };

            if let Some(date) = prev {
                let day = current.day().min(days_in_month(date.year(), date.month()));
                if let Some(final_date) = NaiveDate::from_ymd_opt(date.year(), date.month(), day) {
                    self.selected_date = final_date.max(year_start);
                }
            }
        }
        self
    }

    /// Move selection to the next month.
    #[must_use]
    pub fn next_month(self) -> Self {
        self.next_month_n(1)
    }

    /// Move selection to the previous month.
    #[must_use]
    pub fn prev_month(self) -> Self {
        self.prev_month_n(1)
    }

    /// Move to the next n years.
    #[must_use]
    pub fn next_year_n(self, n: u32) -> Self {
        let year = self.view_year + n as i32;
        self.with_view_year(year)
    }

    /// Move to the previous n years.
    #[must_use]
    pub fn prev_year_n(self, n: u32) -> Self {
        let year = self.view_year - n as i32;
        self.with_view_year(year)
    }

    /// Move to the next year.
    #[must_use]
    pub fn next_year(self) -> Self {
        self.next_year_n(1)
    }

    /// Move to the previous year.
    #[must_use]
    pub fn prev_year(self) -> Self {
        self.prev_year_n(1)
    }

    /// Jump to today.
    #[must_use]
    pub fn go_to_today(self) -> Self {
        let today = self.today;
        let year = today.year();
        self.with_selected_date(today).with_view_year(year)
    }

    // === Month View Navigation ===

    /// Set the month pane focus.
    #[must_use]
    pub fn with_month_pane(mut self, pane: MonthPane) -> Self {
        self.month_pane = pane;
        self
    }

    /// Toggle between days and details panes.
    #[must_use]
    pub fn toggle_pane(mut self) -> Self {
        self.month_pane = match self.month_pane {
            MonthPane::Days => MonthPane::Details,
            MonthPane::Details => MonthPane::Days,
        };
        self
    }

    /// Set the details focus.
    #[must_use]
    pub fn with_details_focus(mut self, focus: DetailsFocus) -> Self {
        self.details_focus = focus;
        self
    }

    /// Cycle to the next section in the details pane.
    #[must_use]
    pub fn next_section(mut self) -> Self {
        self.details_focus = self.details_focus.next();
        self
    }

    /// Cycle to the previous section in the details pane.
    #[must_use]
    pub fn prev_section(mut self) -> Self {
        self.details_focus = self.details_focus.prev();
        self
    }

    /// Move to the next item in the focused section.
    #[must_use]
    pub fn next_item(mut self) -> Self {
        match self.details_focus {
            DetailsFocus::Events => {
                let max = self.current_event_count().saturating_sub(1);
                self.selected_event_idx = self.selected_event_idx.saturating_add(1).min(max);
            }
            DetailsFocus::Goals => {
                let max = self.current_goal_count().saturating_sub(1);
                self.selected_goal_idx = self.selected_goal_idx.saturating_add(1).min(max);
            }
            DetailsFocus::Intention => {
                let max = self.current_intention_count().saturating_sub(1);
                self.selected_intention_idx = self.selected_intention_idx.saturating_add(1).min(max);
            }
        }
        self
    }

    /// Move to the previous item in the focused section.
    #[must_use]
    pub fn prev_item(mut self) -> Self {
        match self.details_focus {
            DetailsFocus::Events => {
                self.selected_event_idx = self.selected_event_idx.saturating_sub(1);
            }
            DetailsFocus::Goals => {
                self.selected_goal_idx = self.selected_goal_idx.saturating_sub(1);
            }
            DetailsFocus::Intention => {
                self.selected_intention_idx = self.selected_intention_idx.saturating_sub(1);
            }
        }
        self
    }

    /// Reset month view state when entering month view.
    #[must_use]
    pub fn reset_month_state(mut self) -> Self {
        self.month_pane = MonthPane::Days;
        self.details_focus = DetailsFocus::Events;
        self.selected_event_idx = 0;
        self.selected_goal_idx = 0;
        self.selected_intention_idx = 0;
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
        self
    }

    /// Set the selected event index.
    #[must_use]
    pub fn with_selected_event_idx(mut self, idx: usize) -> Self {
        self.selected_event_idx = idx;
        self
    }

    /// Get or load month data for the current month.
    pub fn get_month_data(&mut self, year: i32, month: u32) -> &MonthData {
        use crate::data::sample_data;
        self.month_data
            .entry((year, month))
            .or_insert_with(|| sample_data(year, month))
    }

    /// Get month data for the currently selected date's month.
    pub fn current_month_data(&mut self) -> &MonthData {
        let year = self.selected_date.year();
        let month = self.selected_date.month();
        self.get_month_data(year, month)
    }

    /// Get the number of events for the currently selected day.
    fn current_event_count(&self) -> usize {
        let year = self.selected_date.year();
        let month = self.selected_date.month();
        let day = self.selected_date.day();
        self.month_data
            .get(&(year, month))
            .map(|d| d.event_count(day))
            .unwrap_or(0)
    }

    /// Get the number of goals for the current month.
    fn current_goal_count(&self) -> usize {
        let year = self.selected_date.year();
        let month = self.selected_date.month();
        self.month_data
            .get(&(year, month))
            .map(|d| d.goals.len())
            .unwrap_or(0)
    }

    /// Get the number of intentions for the current month.
    fn current_intention_count(&self) -> usize {
        let year = self.selected_date.year();
        let month = self.selected_date.month();
        self.month_data
            .get(&(year, month))
            .map(|d| d.intentions.len())
            .unwrap_or(0)
    }

    // === Input Mode Methods ===

    /// Set the input mode.
    #[must_use]
    pub fn with_input_mode(mut self, mode: InputMode) -> Self {
        self.input_mode = mode;
        self
    }

    /// Open the context menu for the current section.
    #[must_use]
    pub fn open_popup(mut self) -> Self {
        self.input_mode = InputMode::ContextMenu;
        self.menu_action = MenuAction::Add;
        self
    }

    /// Navigate to the next menu action.
    #[must_use]
    pub fn menu_next(mut self) -> Self {
        self.menu_action = self.menu_action.next();
        self
    }

    /// Navigate to the previous menu action.
    #[must_use]
    pub fn menu_prev(mut self) -> Self {
        self.menu_action = self.menu_action.prev();
        self
    }

    /// Check if the current section has a selected item.
    pub fn has_selected_item(&self) -> bool {
        let year = self.selected_date.year();
        let month = self.selected_date.month();
        let day = self.selected_date.day();

        match self.details_focus {
            DetailsFocus::Events => self
                .month_data
                .get(&(year, month))
                .map(|d| d.event_count(day) > 0)
                .unwrap_or(false),
            DetailsFocus::Goals => self
                .month_data
                .get(&(year, month))
                .map(|d| !d.goals.is_empty())
                .unwrap_or(false),
            DetailsFocus::Intention => self
                .month_data
                .get(&(year, month))
                .map(|d| !d.intentions.is_empty())
                .unwrap_or(false),
        }
    }

    /// Select the current menu action and dispatch.
    #[must_use]
    pub fn select_menu_action(mut self) -> Self {
        match self.menu_action {
            MenuAction::Add => self.start_add(),
            MenuAction::Edit => {
                if self.has_selected_item() {
                    self.start_edit()
                } else {
                    self // No item to edit
                }
            }
            MenuAction::Delete => {
                if self.has_selected_item() {
                    // Show confirmation dialog
                    self.input_mode = InputMode::ConfirmDelete;
                    self
                } else {
                    self // No item to delete
                }
            }
        }
    }

    /// Confirm the delete action.
    #[must_use]
    pub fn confirm_delete(self) -> Self {
        self.delete_selected()
    }

    /// Start adding a new item.
    #[must_use]
    fn start_add(mut self) -> Self {
        match self.details_focus {
            DetailsFocus::Goals => {
                self.input_mode = InputMode::AddingGoal;
                self.input_buffer.clear();
            }
            DetailsFocus::Intention => {
                self.input_mode = InputMode::AddingIntention;
                self.input_buffer.clear();
            }
            DetailsFocus::Events => {
                self.input_mode = InputMode::AddingEvent;
                self.event_form_field = EventFormField::Title;
                self.event_title.clear();
                self.event_all_day = false;
                self.event_time.clear();
                self.event_description.clear();
                self.editing_item_id = None;
            }
        }
        self
    }

    /// Start editing the selected item.
    #[must_use]
    fn start_edit(mut self) -> Self {
        let year = self.selected_date.year();
        let month = self.selected_date.month();
        let day = self.selected_date.day();

        match self.details_focus {
            DetailsFocus::Goals => {
                if let Some(data) = self.month_data.get(&(year, month)) {
                    if let Some(goal) = data.goals.get(self.selected_goal_idx) {
                        self.editing_item_id = Some(goal.id);
                        self.input_buffer = goal.title.clone();
                        self.input_mode = InputMode::EditingGoal;
                    }
                }
            }
            DetailsFocus::Intention => {
                if let Some(data) = self.month_data.get(&(year, month)) {
                    if let Some(intention) = data.intentions.get(self.selected_intention_idx) {
                        self.editing_item_id = Some(intention.id);
                        self.input_buffer = intention.text.clone();
                        self.input_mode = InputMode::EditingIntention;
                    }
                }
            }
            DetailsFocus::Events => {
                if let Some(data) = self.month_data.get(&(year, month)) {
                    if let Some(events) = data.events_by_day.get(&day) {
                        if let Some(event) = events.get(self.selected_event_idx) {
                            self.editing_item_id = Some(event.id);
                            self.event_title = event.title.clone();
                            self.event_all_day = event.time.is_none();
                            self.event_time = event
                                .time
                                .map(|t| format!("{:02}{:02}", t.hour(), t.minute()))
                                .unwrap_or_default();
                            self.event_description =
                                event.description.clone().unwrap_or_default();
                            self.event_form_field = EventFormField::Title;
                            self.input_mode = InputMode::EditingEvent;
                        }
                    }
                }
            }
        }
        self
    }

    /// Delete the selected item.
    #[must_use]
    fn delete_selected(mut self) -> Self {
        let year = self.selected_date.year();
        let month = self.selected_date.month();
        let day = self.selected_date.day();

        if let Some(data) = self.month_data.get_mut(&(year, month)) {
            match self.details_focus {
                DetailsFocus::Goals => {
                    if self.selected_goal_idx < data.goals.len() {
                        data.goals.remove(self.selected_goal_idx);
                        // Adjust selection
                        if self.selected_goal_idx > 0 && self.selected_goal_idx >= data.goals.len()
                        {
                            self.selected_goal_idx = data.goals.len().saturating_sub(1);
                        }
                    }
                }
                DetailsFocus::Intention => {
                    if self.selected_intention_idx < data.intentions.len() {
                        data.intentions.remove(self.selected_intention_idx);
                        // Adjust selection
                        if self.selected_intention_idx > 0
                            && self.selected_intention_idx >= data.intentions.len()
                        {
                            self.selected_intention_idx = data.intentions.len().saturating_sub(1);
                        }
                    }
                }
                DetailsFocus::Events => {
                    if let Some(events) = data.events_by_day.get_mut(&day) {
                        if self.selected_event_idx < events.len() {
                            events.remove(self.selected_event_idx);
                            // Adjust selection
                            if self.selected_event_idx > 0
                                && self.selected_event_idx >= events.len()
                            {
                                self.selected_event_idx = events.len().saturating_sub(1);
                            }
                        }
                    }
                }
            }
        }

        self.close_popup()
    }

    /// Close the popup without saving.
    #[must_use]
    pub fn close_popup(mut self) -> Self {
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
        self.menu_action = MenuAction::Add;
        self.editing_item_id = None;
        // Also clear event form
        self.event_form_field = EventFormField::Title;
        self.event_title.clear();
        self.event_all_day = false;
        self.event_time.clear();
        self.event_description.clear();
        self
    }

    /// Push a character to the input buffer.
    #[must_use]
    pub fn push_input_char(mut self, c: char) -> Self {
        self.input_buffer.push(c);
        self
    }

    /// Remove the last character from the input buffer.
    #[must_use]
    pub fn pop_input_char(mut self) -> Self {
        self.input_buffer.pop();
        self
    }

    /// Submit the current input and add to the appropriate list.
    #[must_use]
    pub fn submit_input(mut self) -> Self {
        if self.input_buffer.trim().is_empty() {
            // Don't add empty items
            return self.close_popup();
        }

        let year = self.selected_date.year();
        let month = self.selected_date.month();

        // Ensure month data exists
        if !self.month_data.contains_key(&(year, month)) {
            self.month_data.insert((year, month), crate::data::sample_data(year, month));
        }

        if let Some(data) = self.month_data.get_mut(&(year, month)) {
            match self.input_mode {
                InputMode::AddingGoal => {
                    let new_id = data.goals.iter().map(|g| g.id).max().unwrap_or(0) + 1;
                    data.goals.push(crate::data::Goal {
                        id: new_id,
                        title: self.input_buffer.trim().to_string(),
                        description: None,
                        completed: false,
                    });
                    // Select the new goal
                    self.selected_goal_idx = data.goals.len().saturating_sub(1);
                }
                InputMode::EditingGoal => {
                    if let Some(id) = self.editing_item_id {
                        if let Some(goal) = data.goals.iter_mut().find(|g| g.id == id) {
                            goal.title = self.input_buffer.trim().to_string();
                        }
                    }
                }
                InputMode::AddingIntention => {
                    let new_id = data.intentions.iter().map(|i| i.id).max().unwrap_or(0) + 1;
                    data.intentions.push(crate::data::Intention {
                        id: new_id,
                        text: self.input_buffer.trim().to_string(),
                    });
                    // Select the new intention
                    self.selected_intention_idx = data.intentions.len().saturating_sub(1);
                }
                InputMode::EditingIntention => {
                    if let Some(id) = self.editing_item_id {
                        if let Some(intention) = data.intentions.iter_mut().find(|i| i.id == id) {
                            intention.text = self.input_buffer.trim().to_string();
                        }
                    }
                }
                InputMode::Normal | InputMode::ContextMenu | InputMode::ConfirmDelete | InputMode::AddingEvent | InputMode::EditingEvent => {}
            }
        }

        self.close_popup()
    }

    // === Event Form Methods ===

    /// Move to the next field in the event form.
    #[must_use]
    pub fn event_form_next_field(mut self) -> Self {
        self.event_form_field = match self.event_form_field {
            EventFormField::Title => EventFormField::AllDay,
            EventFormField::AllDay => {
                if self.event_all_day {
                    EventFormField::Description // Skip time if all-day
                } else {
                    EventFormField::Time
                }
            }
            EventFormField::Time => EventFormField::Description,
            EventFormField::Description => EventFormField::Title, // Wrap around
        };
        self
    }

    /// Move to the previous field in the event form.
    #[must_use]
    pub fn event_form_prev_field(mut self) -> Self {
        self.event_form_field = match self.event_form_field {
            EventFormField::Title => EventFormField::Description, // Wrap around
            EventFormField::AllDay => EventFormField::Title,
            EventFormField::Time => EventFormField::AllDay,
            EventFormField::Description => {
                if self.event_all_day {
                    EventFormField::AllDay // Skip time if all-day
                } else {
                    EventFormField::Time
                }
            }
        };
        self
    }

    /// Toggle the all-day checkbox.
    #[must_use]
    pub fn toggle_all_day(mut self) -> Self {
        self.event_all_day = !self.event_all_day;
        if self.event_all_day {
            self.event_time.clear(); // Clear time when switching to all-day
        }
        self
    }

    /// Push a character to the current event form field.
    #[must_use]
    pub fn push_event_char(mut self, c: char) -> Self {
        match self.event_form_field {
            EventFormField::Title => self.event_title.push(c),
            EventFormField::Time => {
                // Only allow digits, max 4 chars (HHMM format)
                if c.is_ascii_digit() && self.event_time.len() < 4 {
                    self.event_time.push(c);
                }
            }
            EventFormField::Description => self.event_description.push(c),
            EventFormField::AllDay => {} // Toggle handles this
        }
        self
    }

    /// Remove the last character from the current event form field.
    #[must_use]
    pub fn pop_event_char(mut self) -> Self {
        match self.event_form_field {
            EventFormField::Title => {
                self.event_title.pop();
            }
            EventFormField::Time => {
                self.event_time.pop();
            }
            EventFormField::Description => {
                self.event_description.pop();
            }
            EventFormField::AllDay => {} // Toggle handles this
        }
        self
    }

    /// Submit the event form and create or update the event.
    #[must_use]
    pub fn submit_event(mut self) -> Self {
        // Title is required
        if self.event_title.trim().is_empty() {
            return self; // Don't submit without title
        }

        let year = self.selected_date.year();
        let month = self.selected_date.month();
        let day = self.selected_date.day();

        // Ensure month data exists
        if !self.month_data.contains_key(&(year, month)) {
            self.month_data
                .insert((year, month), crate::data::sample_data(year, month));
        }

        // Parse time if not all-day
        let time = if self.event_all_day {
            None
        } else {
            parse_time(&self.event_time)
        };

        let is_editing = self.input_mode == InputMode::EditingEvent;

        if let Some(data) = self.month_data.get_mut(&(year, month)) {
            if is_editing {
                // Update existing event
                if let Some(id) = self.editing_item_id {
                    if let Some(events) = data.events_by_day.get_mut(&day) {
                        if let Some(event) = events.iter_mut().find(|e| e.id == id) {
                            event.title = self.event_title.trim().to_string();
                            event.description = if self.event_description.trim().is_empty() {
                                None
                            } else {
                                Some(self.event_description.trim().to_string())
                            };
                            event.time = time;
                        }
                        // Re-sort events by time
                        events.sort_by_key(|e| e.time);
                    }
                }
            } else {
                // Generate new ID for new event
                let new_id = data
                    .events_by_day
                    .values()
                    .flat_map(|events| events.iter().map(|e| e.id))
                    .max()
                    .unwrap_or(0)
                    + 1;

                let event = crate::data::Event {
                    id: new_id,
                    title: self.event_title.trim().to_string(),
                    description: if self.event_description.trim().is_empty() {
                        None
                    } else {
                        Some(self.event_description.trim().to_string())
                    },
                    time,
                    date: self.selected_date,
                };

                // Add to the appropriate day
                data.events_by_day.entry(day).or_default().push(event);

                // Sort events for the day by time
                if let Some(events) = data.events_by_day.get_mut(&day) {
                    events.sort_by_key(|e| e.time);
                }

                // Select the new event
                self.selected_event_idx = data.event_count(day).saturating_sub(1);
            }
        }

        self.close_popup()
    }
}

/// Parse a time string in HHMM format (4 digits).
fn parse_time(s: &str) -> Option<chrono::NaiveTime> {
    let s = s.trim();
    if s.is_empty() || s.len() != 4 {
        return None;
    }
    // Parse HHMM format (e.g., "1430" → 14:30)
    let hours: u32 = s[0..2].parse().ok()?;
    let minutes: u32 = s[2..4].parse().ok()?;
    if hours < 24 && minutes < 60 {
        return chrono::NaiveTime::from_hms_opt(hours, minutes, 0);
    }
    None
}

/// Get the number of days in a month.
fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

/// Check if a year is a leap year.
fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_model_has_today() {
        let model = PlannerModel::default();
        assert_eq!(model.selected_date, model.today);
    }

    #[test]
    fn next_day_increments() {
        let model = PlannerModel::default()
            .with_view_year(2026)
            .with_selected_date(NaiveDate::from_ymd_opt(2026, 1, 15).unwrap());
        let model = model.next_day_n(1);
        assert_eq!(model.selected_date.day(), 16);
    }

    #[test]
    fn prev_day_decrements() {
        let model = PlannerModel::default()
            .with_view_year(2026)
            .with_selected_date(NaiveDate::from_ymd_opt(2026, 1, 15).unwrap());
        let model = model.prev_day_n(1);
        assert_eq!(model.selected_date.day(), 14);
    }
}
