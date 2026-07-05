use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use crate::data::models::*;
use super::state::AppState;

/// Translate crossterm key events into AppEvents and dispatch to the state.
/// Returns a list of AppCommands to execute (side effects).
pub fn handle_key(key: KeyEvent, state: &mut AppState) -> Vec<AppCommand> {
    // Ctrl+C → quit
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        state.running = false;
        return vec![];
    }

    // ? → help (toggle modal)
    if key.code == KeyCode::Char('?') {
        if state.show_modal {
            state.show_modal = false;
        } else {
            state.show_modal = true;
            state.modal_message = "Keyboard shortcuts:\n\
            1-7: Switch tabs\n\
            j/k or ↑/↓: Navigate lists\n\
            Enter: Select / View detail\n\
            b: Paper Buy\n\
            s: Paper Sell\n\
            w: Watchlist toggle\n\
            r: Refresh data\n\
            Esc: Close modal / Back\n\
            Ctrl+E: Emergency exit all\n\
            Tab: Next tab\n\
            q: Quit".to_string();
        }
        return vec![];
    }

    // Tab switching
    if let Some(tab) = match key.code {
        KeyCode::Char('1') => Some(TabIndex::Dashboard),
        KeyCode::Char('2') => Some(TabIndex::Scanner),
        KeyCode::Char('3') => Some(TabIndex::Analyzer),
        KeyCode::Char('4') => Some(TabIndex::TradeTerminal),
        KeyCode::Char('5') => Some(TabIndex::Journal),
        KeyCode::Char('6') => Some(TabIndex::Strategy),
        KeyCode::Char('7') => Some(TabIndex::Settings),
        _ => None,
    } {
        state.switch_tab(tab);
        return vec![];
    }

    // Tab / Shift+Tab for next/prev tab
    if key.code == KeyCode::Tab {
        if key.modifiers.contains(KeyModifiers::SHIFT) {
            state.switch_tab(state.active_tab.prev());
        } else {
            state.switch_tab(state.active_tab.next());
        }
        return vec![];
    }

    // Esc: dismiss modal / deselect
    if key.code == KeyCode::Esc {
        if state.show_modal {
            state.dismiss_modal();
        } else {
            state.selected_token = None;
            state.selected_position_id = None;
        }
        return vec![];
    }

    // j/k or arrow keys: navigate lists
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            state.move_cursor(1);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.move_cursor(-1);
        }
        _ => {}
    }

    // Enter: select current item
    if key.code == KeyCode::Enter {
        match state.active_tab {
            TabIndex::Scanner => {
                // Select token from trending list
                if let Some(token) = state.trending.get(state.list_cursor) {
                    let addr = token.address.clone();
                    let symbol = token.symbol.clone();
                    state.set_status(&format!("Loading {}...", symbol));
                    return vec![AppCommand::FetchTokenDetail(addr)];
                }
            }
            TabIndex::TradeTerminal => {
                // Select position
                if let Some(pos) = state.open_positions.get(state.list_cursor) {
                    state.selected_position_id = Some(pos.id.clone());
                }
            }
            _ => {}
        }
    }

    // r: refresh data
    if key.code == KeyCode::Char('r') {
        state.set_status("Refreshing data...");
        return vec![AppCommand::FetchTrending];
    }

    // q: quit
    if key.code == KeyCode::Char('q') {
        state.running = false;
        return vec![];
    }

    // Ctrl+E: emergency exit all
    if key.code == KeyCode::Char('e') && key.modifiers.contains(KeyModifiers::CONTROL) {
        if state.show_modal && state.modal_message.contains("Emergency Exit") {
            return vec![AppCommand::EmergencyExitAll];
        } else {
            state.show_modal = true;
            state.modal_message = "⚠️  Emergency Exit All\n\nClose ALL open positions at market price?\n\nPress ESC to cancel, ENTER to confirm.".to_string();
        }
        return vec![];
    }

    vec![]
}

/// Handle mouse events
pub fn handle_mouse(_mouse: MouseEvent, _state: &mut AppState) -> Vec<AppCommand> {
    // TODO: mouse click handling for tab bar, buttons, etc.
    vec![]
}

/// Handle resize events
pub fn handle_resize(w: u16, h: u16, state: &mut AppState) {
    state.terminal_size = (w, h);
}