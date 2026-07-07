use super::state::AppState;
use crate::app::state::TokenListMode;
use crate::data::models::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

/// Translate crossterm key events into AppEvents and dispatch to the state.
/// Returns a list of AppCommands to execute (side effects).
pub fn handle_key(key: KeyEvent, state: &mut AppState) -> Vec<AppCommand> {
    // ── Command palette mode — input overrides everything ──────────
    if state.show_command_palette {
        return handle_palette_key(key, state);
    }

    // Ctrl+C → quit (with confirm if open positions)
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        if state.open_positions.is_empty() {
            state.running = false;
        } else {
            state.show_modal = true;
            state.modal_message =
                "Quit with open positions?\n\nPress ENTER to quit, ESC to cancel.".to_string();
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

    // Ctrl+E → emergency exit all
    if key.code == KeyCode::Char('e') && key.modifiers.contains(KeyModifiers::CONTROL) {
        if state.show_modal && state.modal_message.contains("Emergency Exit") {
            state.dismiss_modal();
            return vec![AppCommand::EmergencyExitAll];
        } else {
            state.show_modal = true;
            state.modal_message = "Emergency Exit All\n\nClose ALL open positions at market price?\n\nPress ENTER to confirm, ESC to cancel.".to_string();
        }
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
                 Arrow Up/Down: Navigate lists\n\
                 Arrow Left/Right: Switch tabs\n\
                 Enter: Select / View detail\n\
                 Tab / Shift+Tab: Next/Prev tab\n\
                 b: Paper Buy (Trade tab)\n\
                 s: Paper Sell (Trade tab)\n\
                 r: Refresh data\n\
                 f: Filter tokens\n\
                 /: Search / Filter\n\
                 Space: Watchlist toggle\n\
                 Esc: Close modal / Back\n\
                 Ctrl+P: Command palette\n\
                 Ctrl+B: Toggle sidebar\n\
                 Ctrl+E: Emergency exit all\n\
                 Ctrl+C: Quit\n\
                 ?: This help",
                state.active_tab.label()
            );
        }
        return vec![];
    }

    // Tab / Shift+Tab → cycle tabs
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
                let input = state.input_buffer.clone();
                state.input_buffer.clear();
                if state.address_search_mode {
                    state.address_search_mode = false;
                    // Look up the address
                    state.set_status(&format!("Looking up {}...", input));
                    state.loading_token_detail = true;
                    return vec![AppCommand::FetchTokenDetail(input)];
                }
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

    // Arrow keys — navigation (NO VIM j/k/h/l)
    match key.code {
        KeyCode::Up => state.move_cursor(-1),
        KeyCode::Down => state.move_cursor(1),
        KeyCode::Left => {
            if state.active_tab == TabIndex::Scanner {
                // Switch list mode within Scanner
                state.list_mode = match state.list_mode {
                    TokenListMode::Trending => TokenListMode::AiRec,
                    TokenListMode::Trenches => TokenListMode::Trending,
                    TokenListMode::Watchlist => TokenListMode::Trenches,
                    TokenListMode::AiRec => TokenListMode::Watchlist,
                };
                state.list_cursor = 0;
                state.scroll_offset = 0;
                // Fetch trenches if switching to that mode and empty
                if state.list_mode == TokenListMode::Trenches && state.trenches.is_empty() {
                    return vec![AppCommand::FetchTrenches("new_creation".to_string())];
                }
            } else {
                state.switch_tab(state.active_tab.prev());
            }
            return vec![];
        }
        KeyCode::Right => {
            if state.active_tab == TabIndex::Scanner {
                // Switch list mode within Scanner
                state.list_mode = match state.list_mode {
                    TokenListMode::Trending => TokenListMode::Trenches,
                    TokenListMode::Trenches => TokenListMode::Watchlist,
                    TokenListMode::Watchlist => TokenListMode::AiRec,
                    TokenListMode::AiRec => TokenListMode::Trending,
                };
                state.list_cursor = 0;
                state.scroll_offset = 0;
                // Fetch trenches if switching to that mode and empty
                if state.list_mode == TokenListMode::Trenches && state.trenches.is_empty() {
                    return vec![AppCommand::FetchTrenches("new_creation".to_string())];
                }
            } else {
                state.switch_tab(state.active_tab.next());
            }
            return vec![];
        }
        KeyCode::PageUp => state.move_cursor(-10),
        KeyCode::PageDown => state.move_cursor(10),
        KeyCode::Home => {
            state.list_cursor = 0;
            state.scroll_offset = 0;
        }
        KeyCode::End => {
            let max = state.trending.len().saturating_sub(1);
            state.list_cursor = max;
        }
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
            // Filter modal handling
            if state.modal_message.starts_with("Filter tokens") {
                state.show_filter = !state.show_filter;
                state.dismiss_modal();
                return vec![];
            }
            // Sort modal handling
            if state.modal_message.starts_with("Sort tokens") {
                state.dismiss_modal();
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
                    state.loading_token_detail = true;
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

    // r → refresh (context-sensitive)
    if key.code == KeyCode::Char('r') {
        state.set_status("Refreshing data...");
        state.loading_trending = true;
        if state.active_tab == TabIndex::Scanner && state.list_mode == TokenListMode::Trenches {
            // Fetch trenches instead of trending
            return vec![AppCommand::FetchTrenches("new_creation".to_string())];
        }
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

    // f → filter modal
    if key.code == KeyCode::Char('f') {
        state.show_modal = true;
        state.modal_message = "Filter tokens\n\nToggle filters to narrow the token list.\nActive filters apply to all views.\n\nPress ENTER to apply, ESC to cancel.".to_string();
        return vec![];
    }

    // q → quit (with confirm if open positions)
    if key.code == KeyCode::Char('q') {
        if state.open_positions.is_empty() {
            state.running = false;
        } else {
            state.show_modal = true;
            state.modal_message =
                "Quit with open positions?\n\nPress ENTER to quit, ESC to cancel.".to_string();
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
            if let Some(cmd) = crate::ui::widgets::command_palette::execute_palette_selection(state)
            {
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
    // ── If modal or palette is open, ignore mouse clicks to prevent ghost clicks ──
    if state.show_modal || state.show_command_palette {
        return vec![];
    }

    match mouse.kind {
        MouseEventKind::Down(_button) => {
            // Sidebar: row 0 is top bar, main area starts at row 1
            // Sidebar column is 0..sidebar_width
            let sb_w = state.sidebar_width();
            if mouse.column < sb_w && mouse.row >= 1 {
                if let Some(tab) = crate::ui::sidebar::sidebar_tab_at(
                    mouse.row,
                    ratatui::layout::Rect::new(0, 1, sb_w, 7),
                ) {
                    state.switch_tab(tab);
                    return vec![];
                }
            }

            // Content list click — only on tabs that display token lists
            if mouse.row >= 2 && mouse.column >= sb_w {
                // Only process list selection for tabs that have a token list
                match state.active_tab {
                    TabIndex::Dashboard | TabIndex::Scanner | TabIndex::Analyzer => {
                        let list_idx = (mouse.row - 2) as usize;
                        state.list_cursor = list_idx;

                        // Get the appropriate data source based on list_mode
                        let tokens: Vec<&TrendingToken> = match state.list_mode {
                            TokenListMode::Trenches => {
                                // Trenches use TrenchToken, not TrendingToken
                                return vec![];
                            }
                            _ => state.current_list(),
                        };

                        if let Some(token) = tokens.get(list_idx) {
                            let addr = token.address.clone();
                            let symbol = token.symbol.clone();
                            state.set_status(&format!("Loading {}...", symbol));
                            return vec![AppCommand::FetchTokenDetail(addr)];
                        }
                    }
                    _ => {
                        // Other tabs (Trade, Journal, etc.) don't have token lists to click
                        // Ignore the click
                    }
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
