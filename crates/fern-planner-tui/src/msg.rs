//! Messages - all possible events in the planner.

use crate::model::{DayPane, DetailsFocus, MonthPane, View, WeekPane};
use chrono::NaiveDate;

/// All possible messages/events in the planner application.
#[derive(Debug, Clone)]
pub enum Msg {
    // === Navigation ===
    /// Move to the next day
    NextDay,
    /// Move to the previous day
    PrevDay,
    /// Move to the next week
    NextWeek,
    /// Move to the previous week
    PrevWeek,
    /// Move to the next month
    NextMonth,
    /// Move to the previous month
    PrevMonth,
    /// Move to the next year
    NextYear,
    /// Move to the previous year
    PrevYear,
    /// Jump to today
    GoToToday,
    /// Select a specific date
    SelectDate(NaiveDate),

    // === View Changes ===
    /// Drill down into selected item (year→month, month→day, etc.)
    DrillDown,
    /// Go back up a level (day→month→year)
    DrillUp,
    /// Set the view directly
    SetView(View),

    // === Month View Navigation ===
    /// Toggle between days and details panes (Tab)
    TogglePane,
    /// Set the pane directly
    SetPane(MonthPane),
    /// Move to the next item in the focused section (j in details)
    NextItem,
    /// Move to the previous item in the focused section (k in details)
    PrevItem,
    /// Cycle to the next section (])
    NextSection,
    /// Cycle to the previous section ([)
    PrevSection,
    /// Set the details focus directly
    SetDetailsFocus(DetailsFocus),

    // === Day View Navigation ===
    /// Toggle between timeline and details panes in day view (Tab)
    ToggleDayPane,
    /// Set the day pane directly
    SetDayPane(DayPane),
    /// Move up in timeline (k in timeline pane)
    TimelineUp,
    /// Move down in timeline (j in timeline pane)
    TimelineDown,
    /// Jump to current hour (g in timeline pane)
    GoToCurrentHour,

    // === Week View Navigation ===
    /// Toggle between grid and details panes in week view (Tab)
    ToggleWeekPane,
    /// Set the week pane directly
    SetWeekPane(WeekPane),
    /// Move up in week grid (k in grid pane)
    WeekGridUp,
    /// Move down in week grid (j in grid pane)
    WeekGridDown,
    /// Move left in week grid (h in grid pane)
    WeekGridLeft,
    /// Move right in week grid (l in grid pane)
    WeekGridRight,
    /// Navigate to next week note
    NextWeekNote,
    /// Navigate to previous week note
    PrevWeekNote,
    /// Add a new week note (n)
    AddWeekNote,
    /// Edit the selected week note (e)
    EditWeekNote,
    /// Delete the selected week note (d)
    DeleteWeekNote,
    /// Submit the week note (Enter in popup)
    SubmitWeekNote,
    /// View the selected week note in full (Enter/v)
    ViewWeekNote,

    // === Popup / Input ===
    /// Open popup for adding to current section (Space)
    OpenPopup,
    /// Close popup without saving (Esc in popup)
    ClosePopup,
    /// Submit input and close popup (Enter in popup)
    SubmitInput,
    /// Append a character to input buffer
    InputChar(char),
    /// Remove last character from input buffer
    InputBackspace,

    // === Context Menu ===
    /// Navigate to next menu option (j/Down)
    MenuNext,
    /// Navigate to previous menu option (k/Up)
    MenuPrev,
    /// Select the current menu option (Enter)
    MenuSelect,
    /// Confirm delete action (Enter/y in confirm dialog)
    ConfirmDelete,

    // === Event Form ===
    /// Move to next field in event form (Tab)
    EventFormNextField,
    /// Move to previous field in event form (Shift+Tab)
    EventFormPrevField,
    /// Toggle the all-day checkbox (Space on AllDay field)
    ToggleAllDay,
    /// Toggle start time AM/PM (Space on StartAmPm field)
    ToggleStartAmPm,
    /// Toggle end time AM/PM (Space on EndAmPm field)
    ToggleEndAmPm,
    /// Append a character to current event form field
    EventInputChar(char),
    /// Remove last character from current event form field
    EventInputBackspace,
    /// Submit the event form and create event (Enter)
    SubmitEvent,

    // === Count Prefix (Kakoune-style) ===
    /// Accumulate a digit for count prefix (e.g., "5j" moves 5 days)
    Digit(u32),

    // === System ===
    /// Timer tick (for clock updates)
    Tick,
    /// Terminal resized
    Resize(u16, u16),
    /// Quit the application
    Quit,
}
