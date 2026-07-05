use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use crate::data::models::*;
use super::state::AppState;

/// Translate crossterm key events into AppEvents and dispatch to the state.
/// Returns a list of AppCommands to execute (side effects).
pub fn handle_key(key: KeyEvent, state: &mut AppState) -> Vec<AppCommand> {
    // Ctrl+C → quit
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        if state.open_positions.is_empty() {
            state.running = false;
        } else {
            state.show_modal = true;
            state.modal_message = "Quit with open positions?\n\nPress ENTER to quit, ESC to cancel.".to_string();
        }
        return vec![];
    }

    // ? → help (context-sensitive modal)
    if key.code == KeyCode::Char('?') {
        if state.show_modal {
            state.dismiss_modal();
        } else {
            state.show_modal = true;
            state.modal_message = format!(
                "QuickScope Help — {} Tab\n\n\
                 Keyboard shortcuts:\n\
                 1-7: Switch tabs\n\
                 j/k or ↑/↓: Navigate lists\n\
                 h/l or ←/→: Panel focus\n\
                 Enter: Select / View detail\n\
                 b: Paper Buy | s: Paper Sell\n\
                 w: Watchlist toggle\n\
                 r: Refresh data\n\
                 /: Search / Filter\n\
                 Space: Star / Watch\n\
                 m: Open modal\n\
                 Esc: Close modal / Back\n\
                 Tab: Next tab | Shift+Tab: Prev tab\n\
                 Ctrl+E: Emergency exit all\n\
                 q or Ctrl+C: Quit\n\
                 ?: This help",
                state.active_tab.label()
            );
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

    // Tab / Shift+Tab
    if key.code == KeyCode::Tab {
        if key.modifiers.contains(KeyModifiers::SHIFT) {
            state.switch_tab(state.active_tab.prev());
        } else {
            state.switch_tab(state.active_tab.next());
        }
        return vec![];
    }

    // Esc: dismiss modal / deselect / cancel search
    if key.code == KeyCode::Esc {
        if state.show_modal {
            state.dismiss_modal();
        } else if state.input_active {
            state.input_active = false;
            state.input_buffer.clear();
        } else {
            state.selected_token = None;
            state.selected_position_id = None;
        }
        return vec![];
    }

    // If search input active, handle text entry
    if state.input_active {
        match key.code {
            KeyCode::Char(c) => {
                state.input_buffer.push(c);
            }
            KeyCode::Backspace => {
                state.input_buffer.pop();
            }
            KeyCode::Enter => {
                state.input_active = false;
                // Trigger search based on active tab
                return match state.active_tab {
                    TabIndex::Scanner | TabIndex::Dashboard => {
                        vec![AppCommand::FetchTrending] // will filter client-side
                    }
                    _ => vec![],
                };
            }
            _ => {}
        }
        return vec![];
    }

    // / → activate search
    if key.code == KeyCode::Char('/') {
        state.input_active = true;
        state.input_buffer.clear();
        state.set_status("Search: type query, Enter to submit, Esc to cancel");
        return vec![];
    }

    // Space → toggle watchlist
    if key.code == KeyCode::Char(' ') {
        if let Some(token) = state.trending.get(state.list_cursor) {
            let addr = token.address.clone();
            return vec![AppCommand::AddToWatchlist(addr)];
        }
        return vec![];
    }

    // m → open modal
    if key.code == KeyCode::Char('m') {
        state.show_modal = true;
        state.modal_message = "Quick Actions\n\n\
            a: Analyze selected token\n\
            b: Paper Buy\n\
            s: Paper Sell\n\
            w: Toggle Watchlist\n\
            r: Refresh all data".to_string();
        return vec![];
    }

    // h/l or ←/→ → panel focus
    if key.code == KeyCode::Char('h') || key.code == KeyCode::Left {
        state.set_status("← panel left");
        return vec![];
    }
    if key.code == KeyCode::Char('l') || key.code == KeyCode::Right {
        state.set_status("→ panel right");
        return vec![];
    }

    // j/k or ↑/↓: navigate lists
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => state.move_cursor(1),
        KeyCode::Char('k') | KeyCode::Up => state.move_cursor(-1),
        _ => {}
    }

    // Enter: select / confirm modal
    if key.code == KeyCode::Enter {
        if state.show_modal {
            if state.modal_message.contains("Emergency Exit") {
                state.dismiss_modal();
                return vec![AppCommand::EmergencyExitAll];
            }
            if state.modal_message.contains("Quit with open") {
                state.running = false;
                return vec![];
            }
            state.dismiss_modal();
            return vec![];
        }

        match state.active_tab {
            TabIndex::Scanner => {
                if let Some(token) = state.trending.get(state.list_cursor) {
                    let addr = token.address.clone();
                    let symbol = token.symbol.clone();
                    state.set_status(&format!("Loading {}...", symbol));
                    return vec![AppCommand::FetchTokenDetail(addr)];
                }
            }
            TabIndex::TradeTerminal => {
                if let Some(pos) = state.open_positions.get(state.list_cursor) {
                    state.selected_position_id = Some(pos.id.clone());
                }
            }
            _ => {}
        }
    }

    // r → refresh
    if key.code == KeyCode::Char('r') {
        state.set_status("Refreshing data...");
        return vec![AppCommand::FetchTrending];
    }

    // b → paper buy (Trade tab)
    if key.code == KeyCode::Char('b') && state.active_tab == TabIndex::TradeTerminal {
        state.set_status("Paper Buy — use the order form in the Trade tab");
        return vec![];
    }

    // s → paper sell (Trade tab)
    if key.code == KeyCode::Char('s') && state.active_tab == TabIndex::TradeTerminal {
        state.set_status("Paper Sell — select a position first");
        return vec![];
    }

    // q → quit
    if key.code == KeyCode::Char('q') {
        if state.open_positions.is_empty() {
            state.running = false;
        } else {
            state.show_modal = true;
            state.modal_message = "Quit with open positions?\n\nPress ENTER to quit, ESC to cancel.".to_string();
        }
        return vec![];
    }

    // Ctrl+E: emergency exit all
    if key.code == KeyCode::Char('e') && key.modifiers.contains(KeyModifiers::CONTROL) {
        if state.show_modal && state.modal_message.contains("Emergency Exit") {
            state.dismiss_modal();
            return vec![AppCommand::EmergencyExitAll];
        } else {
            state.show_modal = true;
            state.modal_message = "⚠️  Emergency Exit All\n\nClose ALL open positions at market price?\n\nPress ENTER to confirm, ESC to cancel.".to_string();
        }
        return vec![];
    }

    vec![]
}

/// Handle mouse events — tab bar clicks, list selection, scroll.
pub fn handle_mouse(mouse: MouseEvent, state: &mut AppState) -> Vec<AppCommand> {
    match mouse.kind {
        MouseEventKind::Down(_button) => {
            // Tab bar: rows 0 is top bar, row 1 is tab bar
            if mouse.row == 1 {
                let tab_idx = (mouse.column.saturating_sub(1) / 10) as usize;
                if tab_idx < TabIndex::COUNT {
                    state.switch_tab(TabIndex::from_usize(tab_idx));
                }
            }
            // List item click: row = 2 (top bar) + 1 (tab bar) + 2 (header) + offset
            if mouse.row >= 5 {
                let list_idx = (mouse.row - 5) as usize;
                state.list_cursor = list_idx;
                // Auto-select on click
                if let Some(token) = state.trending.get(list_idx) {
                    let addr = token.address.clone();
                    let symbol = token.symbol.clone();
                    state.set_status(&format!("Loading {}...", symbol));
                    return vec![AppCommand::FetchTokenDetail(addr)];
                }
            }
        }
        MouseEventKind::ScrollUp => {
            state.move_cursor(-1);
        }
        MouseEventKind::ScrollDown => {
            state.move_cursor(1);
        }
        _ => {}
    }
    vec![]
}

/// Handle resize events
pub fn handle_resize(w: u16, h: u16, state: &mut AppState) {
    state.terminal_size = (w, h);
}