use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use crate::data::models::*;
use super::state::AppState;

/// Translate crossterm key events into AppEvents and dispatch to the state.
/// Returns a list of AppCommands to execute (side effects).
pub fn handle_key(key: KeyEvent, state: &mut AppState) -> Vec<AppCommand> {
    // ── Command palette mode — input overrides everything ──────────
    if state.show_command_palette {
        return handle_palette_key(key, state);
    }

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

    // Ctrl+P → command palette
    if key.code == KeyCode::Char('p') && key.modifiers.contains(KeyModifiers::CONTROL) {
        state.toggle_command_palette();
        return vec![];
    }

    // Ctrl+B → toggle sidebar
    if key.code == KeyCode::Char('b') && key.modifiers.contains(KeyModifiers::CONTROL) {
        state.toggle_sidebar();
        return vec![];
    }

    // ? → help (modal)
    if key.code == KeyCode::Char('?') {
        if state.show_modal {
            state.dismiss_modal();
        } else {
            state.show_modal = true;
            state.modal_message = format!(
                "QuickScope Help — {} Tab\n\n\
                 Keyboard shortcuts:\n\
                 ↑/↓: Navigate lists\n\
                 Enter: Select / View detail\n\
                 b: Paper Buy | s: Paper Sell\n\
                 w: Watchlist toggle\n\
                 r: Refresh data\n\
                 /: Search / Filter\n\
                 Space: Star / Watch\n\
                 Esc: Close modal / Back\n\
                 Tab: Next tab | Shift+Tab: Prev tab\n\
                 Ctrl+P: Command palette\n\
                 Ctrl+B: Toggle sidebar\n\
                 Ctrl+E: Emergency exit all\n\
                 q or Ctrl+C: Quit\n\
                 ?: This help",
                state.active_tab.label()
            );
        }
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

    // Esc: dismiss modal / deselect / cancel search / close palette
    if key.code == KeyCode::Esc {
        if state.show_modal {
            state.dismiss_modal();
        } else if state.show_command_palette {
            state.show_command_palette = false;
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
                return match state.active_tab {
                    TabIndex::Scanner | TabIndex::Dashboard => {
                        vec![AppCommand::FetchTrending]
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

    // h/l or ←/→ → panel focus (future use)
    if key.code == KeyCode::Char('h') || key.code == KeyCode::Left {
        state.set_status("← panel left");
        return vec![];
    }
    if key.code == KeyCode::Char('l') || key.code == KeyCode::Right {
        state.set_status("→ panel right");
        return vec![];
    }

    // ↑/↓: navigate lists (no VIM j/k)
    match key.code {
        KeyCode::Up => state.move_cursor(-1),
        KeyCode::Down => state.move_cursor(1),
        KeyCode::PageUp => state.move_cursor(-10),
        KeyCode::PageDown => state.move_cursor(10),
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

/// Handle key events while the command palette is open.
fn handle_palette_key(key: KeyEvent, state: &mut AppState) -> Vec<AppCommand> {
    match key.code {
        KeyCode::Esc => {
            state.show_command_palette = false;
            vec![]
        }
        KeyCode::Enter => {
            if let Some(cmd) = crate::ui::widgets::command_palette::execute_palette_selection(state) {
                vec![cmd]
            } else {
                vec![]
            }
        }
        KeyCode::Up => {
            state.palette_cursor = state.palette_cursor.saturating_sub(1);
            vec![]
        }
        KeyCode::Down => {
            state.palette_cursor += 1;
            vec![]
        }
        KeyCode::Char(c) => {
            state.palette_filter.push(c);
            state.palette_cursor = 0;
            vec![]
        }
        KeyCode::Backspace => {
            state.palette_filter.pop();
            state.palette_cursor = 0;
            vec![]
        }
        _ => vec![],
    }
}

/// Handle mouse events — sidebar clicks, list selection, scroll.
pub fn handle_mouse(mouse: MouseEvent, state: &mut AppState) -> Vec<AppCommand> {
    match mouse.kind {
        MouseEventKind::Down(_button) => {
            // Sidebar: row 0 is top bar, main area starts at row 1
            // Sidebar column is 0..sidebar_width
            let sb_w = state.sidebar_width();
            if mouse.column < sb_w && mouse.row >= 1 {
                if let Some(tab) = crate::ui::sidebar::sidebar_tab_at(mouse.row, 
                    ratatui::layout::Rect::new(0, 1, sb_w, 7)
                ) {
                    state.switch_tab(tab);
                    return vec![];
                }
            }

            // Content list click
            if mouse.row >= 2 && mouse.column >= sb_w {
                let list_idx = (mouse.row - 2) as usize;
                state.list_cursor = list_idx;
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
