//! Application trait implementation for the planner.

use chrono::Datelike;
use frond::prelude::*;

use crate::model::{EventFormField, InputMode, MonthPane, PlannerModel, View};
use crate::msg::Msg;
use crate::view;

/// The planner application.
pub struct PlannerApp;

impl Application for PlannerApp {
    type Model = PlannerModel;
    type Msg = Msg;

    fn init() -> (Self::Model, Cmd<Self::Msg>) {
        (PlannerModel::new(), Cmd::none())
    }

    fn update(mut model: Self::Model, msg: Self::Msg) -> (Self::Model, Cmd<Self::Msg>) {
        // Ensure month data is loaded when in month view
        if model.view == View::Month {
            let year = model.selected_date.year();
            let month = model.selected_date.month();
            model.get_month_data(year, month);
        }

        let model = match msg {
            // Count prefix (Kakoune-style)
            Msg::Digit(d) => model.push_digit(d),

            // Navigation - uses count then clears it
            Msg::NextDay => {
                let n = model.effective_count();
                let m = model.clear_count().next_day_n(n);
                // Reset event index when day changes
                m.with_selected_event_idx(0)
            }
            Msg::PrevDay => {
                let n = model.effective_count();
                let m = model.clear_count().prev_day_n(n);
                m.with_selected_event_idx(0)
            }
            Msg::NextWeek => {
                let n = model.effective_count();
                let m = model.clear_count().next_week_n(n);
                m.with_selected_event_idx(0)
            }
            Msg::PrevWeek => {
                let n = model.effective_count();
                let m = model.clear_count().prev_week_n(n);
                m.with_selected_event_idx(0)
            }
            Msg::NextMonth => {
                let n = model.effective_count();
                model.clear_count().next_month_n(n).reset_month_state()
            }
            Msg::PrevMonth => {
                let n = model.effective_count();
                model.clear_count().prev_month_n(n).reset_month_state()
            }
            Msg::NextYear => {
                let n = model.effective_count();
                model.clear_count().next_year_n(n)
            }
            Msg::PrevYear => {
                let n = model.effective_count();
                model.clear_count().prev_year_n(n)
            }
            Msg::GoToToday => model.clear_count().go_to_today().reset_month_state(),
            Msg::SelectDate(date) => model.clear_count().with_selected_date(date),

            // View changes - clear count
            Msg::DrillDown => drill_down(model.clear_count()),
            Msg::DrillUp => drill_up(model.clear_count()),
            Msg::SetView(view) => {
                let m = model.clear_count().with_view(view);
                if view == View::Month {
                    m.reset_month_state()
                } else {
                    m
                }
            }

            // Month view navigation
            Msg::TogglePane => model.clear_count().toggle_pane(),
            Msg::SetPane(pane) => model.clear_count().with_month_pane(pane),
            Msg::NextItem => model.clear_count().next_item(),
            Msg::PrevItem => model.clear_count().prev_item(),
            Msg::NextSection => model.clear_count().next_section(),
            Msg::PrevSection => model.clear_count().prev_section(),
            Msg::SetDetailsFocus(focus) => model.clear_count().with_details_focus(focus),

            // Popup / Input
            Msg::OpenPopup => model.open_popup(),
            Msg::ClosePopup => model.close_popup(),
            Msg::SubmitInput => model.submit_input(),
            Msg::InputChar(c) => model.push_input_char(c),
            Msg::InputBackspace => model.pop_input_char(),

            // Context Menu
            Msg::MenuNext => model.menu_next(),
            Msg::MenuPrev => model.menu_prev(),
            Msg::MenuSelect => model.select_menu_action(),
            Msg::ConfirmDelete => model.confirm_delete(),

            // Event Form
            Msg::EventFormNextField => model.event_form_next_field(),
            Msg::EventFormPrevField => model.event_form_prev_field(),
            Msg::ToggleAllDay => model.toggle_all_day(),
            Msg::EventInputChar(c) => model.push_event_char(c),
            Msg::EventInputBackspace => model.pop_event_char(),
            Msg::SubmitEvent => model.submit_event(),

            // System
            Msg::Tick => {
                let today = chrono::Local::now().date_naive();
                model.with_today(today)
            }
            Msg::Resize(_, _) => model, // Re-render will handle it
            Msg::Quit => model.with_should_quit(true),
        };

        (model, Cmd::none())
    }

    fn view(model: &Self::Model, frame: &mut Frame) {
        view::render(model, frame);
    }

    fn subscriptions(model: &Self::Model) -> Sub<Self::Msg> {
        // View-specific key handling
        let view = model.view;
        let month_pane = model.month_pane;
        let input_mode = model.input_mode;
        let event_form_field = model.event_form_field;

        Sub::batch([
            // Keyboard input - view-aware
            Sub::on_key(move |key| {
                use KeyCode::*;

                // Handle context menu
                if input_mode == InputMode::ContextMenu {
                    return match key.code {
                        Esc => Some(Msg::ClosePopup),
                        Enter => Some(Msg::MenuSelect),
                        Char('j') | Down => Some(Msg::MenuNext),
                        Char('k') | Up => Some(Msg::MenuPrev),
                        _ => None,
                    };
                }

                // Handle confirm delete dialog
                if input_mode == InputMode::ConfirmDelete {
                    return match key.code {
                        Esc | Char('n') | Char('N') => Some(Msg::ClosePopup),
                        Enter | Char('y') | Char('Y') => Some(Msg::ConfirmDelete),
                        _ => None,
                    };
                }

                // Handle event form input mode (multi-field form) - both adding and editing
                if input_mode == InputMode::AddingEvent || input_mode == InputMode::EditingEvent {
                    return match key.code {
                        Esc => Some(Msg::ClosePopup),
                        Enter => Some(Msg::SubmitEvent),
                        Tab => Some(Msg::EventFormNextField),
                        BackTab => Some(Msg::EventFormPrevField), // Shift+Tab
                        Backspace => Some(Msg::EventInputBackspace),
                        // Space toggles all-day when on that field
                        Char(' ') if event_form_field == EventFormField::AllDay => {
                            Some(Msg::ToggleAllDay)
                        }
                        Char(c) => Some(Msg::EventInputChar(c)),
                        _ => None,
                    };
                }

                // Handle simple popup input modes (Goal/Intention) - both adding and editing
                if matches!(input_mode, InputMode::AddingGoal | InputMode::AddingIntention
                    | InputMode::EditingGoal | InputMode::EditingIntention) {
                    return match key.code {
                        Esc => Some(Msg::ClosePopup),
                        Enter => Some(Msg::SubmitInput),
                        Backspace => Some(Msg::InputBackspace),
                        Char(c) => Some(Msg::InputChar(c)),
                        _ => None,
                    };
                }

                // Common keys across all views
                match key.code {
                    // Quit
                    Char('q') => return Some(Msg::Quit),

                    // Count prefix (Kakoune-style: type number then movement)
                    Char('1') => return Some(Msg::Digit(1)),
                    Char('2') => return Some(Msg::Digit(2)),
                    Char('3') => return Some(Msg::Digit(3)),
                    Char('4') => return Some(Msg::Digit(4)),
                    Char('5') => return Some(Msg::Digit(5)),
                    Char('6') => return Some(Msg::Digit(6)),
                    Char('7') => return Some(Msg::Digit(7)),
                    Char('8') => return Some(Msg::Digit(8)),
                    Char('9') => return Some(Msg::Digit(9)),
                    Char('0') => return Some(Msg::Digit(0)),

                    // Jump to today (all views)
                    Char('t') => return Some(Msg::GoToToday),

                    _ => {}
                }

                // View-specific keys
                match view {
                    View::Year => match key.code {
                        Esc => Some(Msg::DrillUp), // Quit from year view

                        // Navigation - vim style
                        Char('l') => Some(Msg::NextMonth),
                        Char('h') => Some(Msg::PrevMonth),
                        Char('j') => Some(Msg::NextDay),
                        Char('k') => Some(Msg::PrevDay),

                        // Uppercase = larger jumps
                        Char('L') => Some(Msg::NextYear),
                        Char('H') => Some(Msg::PrevYear),
                        Char('J') => Some(Msg::NextWeek),
                        Char('K') => Some(Msg::PrevWeek),

                        // Arrow keys
                        Right => Some(Msg::NextMonth),
                        Left => Some(Msg::PrevMonth),
                        Down => Some(Msg::NextDay),
                        Up => Some(Msg::PrevDay),

                        Enter => Some(Msg::DrillDown),

                        _ => None,
                    },

                    View::Month => {
                        // Month view navigation depends on which pane is focused
                        match month_pane {
                            MonthPane::Days => match key.code {
                                Esc => Some(Msg::DrillUp), // Back to year view

                                // Navigation within days pane
                                Char('j') | Down => Some(Msg::NextDay),
                                Char('k') | Up => Some(Msg::PrevDay),
                                Char('l') | Right => Some(Msg::NextMonth),
                                Char('h') | Left => Some(Msg::PrevMonth),
                                Char('J') => Some(Msg::NextWeek),
                                Char('K') => Some(Msg::PrevWeek),

                                // Switch to details pane
                                Tab => Some(Msg::TogglePane),

                                // Drill down to day view
                                Enter => Some(Msg::DrillDown),

                                _ => None,
                            },
                            MonthPane::Details => match key.code {
                                Esc => Some(Msg::SetPane(MonthPane::Days)), // Back to days pane

                                // Navigation within current section
                                Char('j') | Down => Some(Msg::NextItem),
                                Char('k') | Up => Some(Msg::PrevItem),

                                // Section cycling (changed from g/G to [/])
                                Char(']') => Some(Msg::NextSection),
                                Char('[') => Some(Msg::PrevSection),

                                // Open popup for adding (Space)
                                Char(' ') => Some(Msg::OpenPopup),

                                // Switch back to days pane
                                Tab => Some(Msg::TogglePane),

                                _ => None,
                            },
                        }
                    }

                    View::Week | View::Day => match key.code {
                        Esc => Some(Msg::DrillUp),

                        // Basic navigation
                        Char('j') | Down => Some(Msg::NextDay),
                        Char('k') | Up => Some(Msg::PrevDay),
                        Char('l') | Right => Some(Msg::NextMonth),
                        Char('h') | Left => Some(Msg::PrevMonth),

                        Enter => Some(Msg::DrillDown),

                        _ => None,
                    },
                }
            }),
            // Update time every minute
            Sub::every(Duration::from_secs(60), Msg::Tick),
        ])
    }

    fn should_quit(model: &Self::Model) -> bool {
        model.should_quit
    }
}

/// Drill down from current view.
fn drill_down(model: PlannerModel) -> PlannerModel {
    match model.view {
        View::Year => model.with_view(View::Month),
        View::Month => model.with_view(View::Day),
        View::Week => model.with_view(View::Day),
        View::Day => model, // Already at deepest level
    }
}

/// Drill up from current view.
fn drill_up(model: PlannerModel) -> PlannerModel {
    match model.view {
        View::Year => model.with_should_quit(true), // Quit from year view
        View::Month => model.with_view(View::Year),
        View::Week => model.with_view(View::Month),
        View::Day => model.with_view(View::Month),
    }
}
