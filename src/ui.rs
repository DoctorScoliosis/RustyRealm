use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io::{self, Stdout};

use crate::state::GameState;

/// Keeps the interactive terminal state scoped to the lifetime of the UI.
struct TerminalGuard;

impl TerminalGuard {
    fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        if let Err(error) = execute!(stdout, EnterAlternateScreen, Hide) {
            let _ = disable_raw_mode();
            return Err(error);
        }
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        // Best-effort restoration also runs while unwinding from a panic.
        let mut stdout = io::stdout();
        let _ = execute!(stdout, Show, LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    _guard: TerminalGuard,
}

impl Tui {
    pub fn new() -> io::Result<Self> {
        let guard = TerminalGuard::new()?;
        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => return Err(error),
        };
        terminal.clear()?;
        Ok(Self { terminal, _guard: guard })
    }

    /// Draw a fresh frame using the terminal's current dimensions.
    pub fn render_screen<F>(&mut self, state: Option<&GameState>, render_main: F) -> io::Result<()>
    where
        F: FnOnce(&mut Frame<'_>, Rect, Option<&GameState>),
    {
        self.terminal.draw(|frame| {
            let area = frame.size();
            render_responsive_base(frame, area, state, render_main);
        })?;
        Ok(())
    }

    /// Arrow-key menu. Resize events trigger a new draw before waiting for input.
    pub fn select_menu(
        &mut self,
        title: &str,
        options: &[&str],
        state: Option<&GameState>,
    ) -> io::Result<usize> {
        if options.is_empty() {
            return Ok(0);
        }

        let mut selected = 0usize;
        loop {
            self.render_screen(state, |frame, area, _| {
                let title_height = if area.height >= 7 { 3 } else { 0 };
                let footer_height = if area.height >= 10 { 2 } else { 0 };
                let content_height = area.height.saturating_sub(title_height + footer_height);
                let mut constraints = Vec::new();
                if title_height > 0 {
                    constraints.push(Constraint::Length(title_height));
                }
                constraints.push(Constraint::Length(content_height));
                if footer_height > 0 {
                    constraints.push(Constraint::Length(footer_height));
                }
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(constraints)
                    .split(area);
                let mut index = 0usize;

                if title_height > 0 {
                    let header = Paragraph::new(title)
                        .alignment(Alignment::Center)
                        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                        .block(Block::default().borders(Borders::ALL));
                    frame.render_widget(header, chunks[index]);
                    index += 1;
                }

                let visible_options: Vec<ListItem<'_>> = options
                    .iter()
                    .enumerate()
                    .map(|(i, option)| {
                        let label = if i < 9 {
                            format!("{}) {}", i + 1, option)
                        } else {
                            format!("   {}", option)
                        };
                        ListItem::new(Line::from(label))
                    })
                    .collect();

                let list = List::new(visible_options)
                    .block(Block::default().title(if title_height == 0 { title } else { " Options " }).borders(Borders::ALL))
                    .highlight_style(
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )
                    .highlight_symbol("▶ ");
                let mut list_state = ListState::default();
                list_state.select(Some(selected.min(options.len().saturating_sub(1))));
                frame.render_stateful_widget(list, chunks[index], &mut list_state);
                index += 1;

                if footer_height > 0 {
                    let footer = Paragraph::new("↑/↓ or j/k: move  ·  Enter: select  ·  Esc: back")
                        .alignment(Alignment::Center)
                        .style(Style::default().fg(Color::DarkGray))
                        .block(Block::default().borders(Borders::ALL));
                    frame.render_widget(footer, chunks[index]);
                }
            })?;

            match event::read()? {
                Event::Resize(_, _) => continue,
                Event::Key(KeyEvent { code: KeyCode::Up, .. })
                | Event::Key(KeyEvent { code: KeyCode::Char('k'), .. }) => {
                    selected = if selected == 0 { options.len() - 1 } else { selected - 1 };
                }
                Event::Key(KeyEvent { code: KeyCode::Down, .. })
                | Event::Key(KeyEvent { code: KeyCode::Char('j'), .. }) => {
                    selected = if selected + 1 >= options.len() { 0 } else { selected + 1 };
                }
                Event::Key(KeyEvent { code: KeyCode::Enter, .. }) => return Ok(selected),
                Event::Key(KeyEvent { code: KeyCode::Esc, .. }) => {
                    let back_index = options.iter().position(|option| {
                        let label = option.to_ascii_lowercase();
                        label.contains("back")
                            || label.contains("exit")
                            || label.contains("leave")
                            || label.contains("return")
                            || label.contains("flee")
                            || label.contains("cancel")
                    }).unwrap_or(options.len() - 1);
                    return Ok(back_index);
                },
                Event::Key(KeyEvent { code: KeyCode::Char(c), .. }) if c.is_ascii_digit() => {
                    let digit = c.to_digit(10).unwrap_or(0) as usize;
                    if digit > 0 && digit <= options.len() && digit <= 9 {
                        return Ok(digit - 1);
                    }
                }
                _ => {}
            }
        }
    }

    pub fn prompt_input(
        &mut self,
        prompt: &str,
        masked: bool,
        state: Option<&GameState>,
    ) -> io::Result<String> {
        let mut input = String::new();
        loop {
            self.render_screen(state, |frame, area, _| {
                let popup = centered_rect(88, 35, area);
                frame.render_widget(Clear, popup);
                let block = Block::default()
                    .title(format!(" {} ", prompt))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow));
                let inner = block.inner(popup);
                frame.render_widget(block, popup);

                let shown = if masked { "*".repeat(input.chars().count()) } else { input.clone() };
                let paragraph = Paragraph::new(format!("> {}", shown))
                    .style(Style::default().fg(Color::White))
                    .wrap(Wrap { trim: true });
                frame.render_widget(paragraph, inner);

                if inner.width > 0 && inner.height > 0 {
                    let input_cells = if masked { input.chars().count() } else { input.chars().count() };
                    let cursor_x = inner.x.saturating_add(2).saturating_add(input_cells.min(inner.width.saturating_sub(3) as usize) as u16);
                    frame.set_cursor(cursor_x.min(inner.x + inner.width - 1), inner.y);
                }
            })?;

            match event::read()? {
                Event::Resize(_, _) => continue,
                Event::Key(KeyEvent { code: KeyCode::Enter, .. }) => return Ok(input),
                Event::Key(KeyEvent { code: KeyCode::Esc, .. }) => return Ok(String::new()),
                Event::Key(KeyEvent { code: KeyCode::Char(c), .. }) => input.push(c),
                Event::Key(KeyEvent { code: KeyCode::Backspace, .. }) => { input.pop(); }
                _ => {}
            }
        }
    }

    pub fn show_message(
        &mut self,
        title: &str,
        message: &str,
        state: Option<&GameState>,
    ) -> io::Result<()> {
        let mut scroll = 0u16;
        loop {
            self.render_screen(state, |frame, area, _| {
                let popup = centered_rect(92, 82, area);
                frame.render_widget(Clear, popup);
                let block = Block::default()
                    .title(format!(" {} ", title))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green));
                let inner = block.inner(popup);
                frame.render_widget(block, popup);
                let paragraph = Paragraph::new(message)
                    .wrap(Wrap { trim: true })
                    .scroll((scroll, 0));
                frame.render_widget(paragraph, inner);
            })?;

            match event::read()? {
                Event::Resize(_, _) => continue,
                Event::Key(KeyEvent { code: KeyCode::Up, .. }) => scroll = scroll.saturating_sub(1),
                Event::Key(KeyEvent { code: KeyCode::Down, .. }) => scroll = scroll.saturating_add(1),
                Event::Key(KeyEvent { code: KeyCode::PageUp, .. }) => scroll = scroll.saturating_sub(8),
                Event::Key(KeyEvent { code: KeyCode::PageDown, .. }) => scroll = scroll.saturating_add(8),
                Event::Key(KeyEvent { code: KeyCode::Enter, .. })
                | Event::Key(KeyEvent { code: KeyCode::Esc, .. })
                | Event::Key(KeyEvent { code: KeyCode::Char(' '), .. }) => return Ok(()),
                _ => {}
            }
        }
    }

    /// Full-page, scrollable text screen for longer information such as inventory.
    pub fn show_text(
        &mut self,
        title: &str,
        content: &str,
        state: Option<&GameState>,
    ) -> io::Result<()> {
        let mut scroll = 0u16;
        loop {
            self.render_screen(state, |frame, area, _| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Min(1), Constraint::Length(if area.height >= 5 { 2 } else { 0 })])
                    .split(area);
                let header = Paragraph::new(title)
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                    .block(Block::default().borders(Borders::ALL));
                frame.render_widget(header, chunks[0]);
                let body = Paragraph::new(content)
                    .wrap(Wrap { trim: true })
                    .scroll((scroll, 0))
                    .block(Block::default().borders(Borders::LEFT | Borders::RIGHT));
                frame.render_widget(body, chunks[1]);
                if area.height >= 5 {
                    let footer = Paragraph::new("↑/↓ scroll · PgUp/PgDn page · Enter/Esc back")
                        .alignment(Alignment::Center)
                        .style(Style::default().fg(Color::DarkGray))
                        .block(Block::default().borders(Borders::ALL));
                    frame.render_widget(footer, chunks[2]);
                }
            })?;

            match event::read()? {
                Event::Resize(_, _) => continue,
                Event::Key(KeyEvent { code: KeyCode::Up, .. }) => scroll = scroll.saturating_sub(1),
                Event::Key(KeyEvent { code: KeyCode::Down, .. }) => scroll = scroll.saturating_add(1),
                Event::Key(KeyEvent { code: KeyCode::PageUp, .. }) => scroll = scroll.saturating_sub(8),
                Event::Key(KeyEvent { code: KeyCode::PageDown, .. }) => scroll = scroll.saturating_add(8),
                Event::Key(KeyEvent { code: KeyCode::Enter, .. })
                | Event::Key(KeyEvent { code: KeyCode::Esc, .. }) => return Ok(()),
                _ => {}
            }
        }
    }
}

pub fn render_responsive_base<F>(
    frame: &mut Frame<'_>,
    area: Rect,
    state: Option<&GameState>,
    render_main: F,
) where
    F: FnOnce(&mut Frame<'_>, Rect, Option<&GameState>),
{
    let Some(state) = state else {
        render_main(frame, area, None);
        return;
    };

    if area.width >= 90 && area.height >= 20 {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(28), Constraint::Min(1)])
            .split(area);
        render_sidebar(frame, columns[0], state);
        render_main(frame, columns[1], Some(state));
    } else if area.width >= 60 && area.height >= 20 {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(area);
        render_topbar(frame, rows[0], state);
        render_main(frame, rows[1], Some(state));
    } else if area.height >= 5 {
        // Narrow or short terminals use only a single compact status row.
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(area);
        render_compact_header(frame, rows[0], state);
        render_main(frame, rows[1], Some(state));
    } else {
        // At very small heights, preserve the complete content area.
        render_main(frame, area, Some(state));
    }
}

fn render_sidebar(frame: &mut Frame<'_>, area: Rect, state: &GameState) {
    let block = Block::default()
        .title(" Hero ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(3), Constraint::Length(3), Constraint::Min(1)])
        .split(inner);

    frame.render_widget(
        Paragraph::new(format!("{} · Combat {}", state.player.username, state.player.combat_level))
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .wrap(Wrap { trim: true }),
        chunks[0],
    );

    let hp_ratio = if state.player.max_hp > 0 {
        (state.player.hp as f64 / state.player.max_hp as f64).clamp(0.0, 1.0)
    } else { 0.0 };
    frame.render_widget(
        Gauge::default()
            .block(Block::default().title(" HP "))
            .gauge_style(Style::default().fg(Color::Red))
            .ratio(hp_ratio)
            .label(format!("{}/{}", state.player.hp, state.player.max_hp)),
        chunks[1],
    );

    let xp_ratio = if state.player.combat_level >= crate::data::MAX_PLAYER_LEVEL {
        1.0
    } else if state.player.orig_xp_threshold > 0 {
        (1.0 - state.player.xp_until_next as f64 / state.player.orig_xp_threshold as f64).clamp(0.0, 1.0)
    } else { 0.0 };
    frame.render_widget(
        Gauge::default()
            .block(Block::default().title(" EXP "))
            .gauge_style(Style::default().fg(Color::Green))
            .ratio(xp_ratio)
            .label(format!("{} XP", state.player.xp)),
        chunks[2],
    );

    let details = format!(
        "Gold: {}\nWeapon: {}\nArmour: {}\nTotal level: {}\n\nLogs: {}  Ore: {}\nFood: {}  Potions: {}",
        state.player.gold,
        state.player.weapon.name(),
        state.player.armour.name(),
        state.player.skills.total_level() + state.player.combat_level,
        state.player.get_item_count(crate::data::ItemType::Logs),
        state.player.get_item_count(crate::data::ItemType::Ore),
        state.player.get_item_count(crate::data::ItemType::Food),
        state.player.get_item_count(crate::data::ItemType::Potions),
    );
    frame.render_widget(Paragraph::new(details).wrap(Wrap { trim: true }), chunks[3]);
}

fn render_topbar(frame: &mut Frame<'_>, area: Rect, state: &GameState) {
    let text = format!(
        " {}  ·  Lv {}  ·  HP {}/{}  ·  Gold {}  ·  {} ",
        state.player.username,
        state.player.combat_level,
        state.player.hp,
        state.player.max_hp,
        state.player.gold,
        state.player.weapon.name(),
    );
    frame.render_widget(
        Paragraph::new(text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Yellow).bg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL)),
        area,
    );
}

fn render_compact_header(frame: &mut Frame<'_>, area: Rect, state: &GameState) {
    let text = format!(
        "Lv{} HP:{}/{} G:{}",
        state.player.combat_level, state.player.hp, state.player.max_hp, state.player.gold
    );
    frame.render_widget(Paragraph::new(text).style(Style::default().fg(Color::Yellow)), area);
}

pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let percent_x = percent_x.clamp(1, 100);
    let percent_y = percent_y.clamp(1, 100);
    let left = (100 - percent_x) / 2;
    let right = 100 - percent_x - left;
    let top = (100 - percent_y) / 2;
    let bottom = 100 - percent_y - top;

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(top), Constraint::Percentage(percent_y), Constraint::Percentage(bottom)])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(left), Constraint::Percentage(percent_x), Constraint::Percentage(right)])
        .split(vertical[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;

    #[test]
    fn responsive_layout_renders_within_wide_standard_narrow_and_short_terminals() {
        let state = GameState::new(crate::state::Player::new("Hero".into(), String::new()));
        for (width, height) in [(120, 30), (90, 20), (75, 25), (60, 20), (45, 30), (45, 15), (80, 12), (20, 4), (1, 1)] {
            let backend = TestBackend::new(width, height);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.draw(|frame| {
                let full = frame.size();
                render_responsive_base(frame, full, Some(&state), |frame, content, _| {
                    assert!(content.x.saturating_add(content.width) <= full.width);
                    assert!(content.y.saturating_add(content.height) <= full.height);
                    frame.render_widget(Paragraph::new("Responsive content"), content);
                });
            }).unwrap();
        }
    }

    #[test]
    fn centered_rect_stays_inside_small_areas() {
        for (width, height) in [(1, 1), (10, 3), (40, 10), (120, 30)] {
            let area = Rect::new(0, 0, width, height);
            let popup = centered_rect(92, 82, area);
            assert!(popup.x.saturating_add(popup.width) <= area.width);
            assert!(popup.y.saturating_add(popup.height) <= area.height);
        }
    }
}
