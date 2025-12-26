//! Messages - all possible events in the planner.

use crate::model::{DetailsFocus, MonthPane, View};
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
