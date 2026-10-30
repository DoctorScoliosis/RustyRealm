use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyEvent, MouseButton, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io::{self, IsTerminal, Stdout, Write};

use crate::state::GameState;

/// Keeps the interactive terminal state scoped to the lifetime of the UI.
/// Mouse capture is best-effort; the application remains keyboard-operable if enabling it fails.
struct TerminalGuard;

impl TerminalGuard {
    fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        if let Err(error) = execute!(stdout, EnterAlternateScreen, Hide) {
            let _ = execute!(stdout, Show, LeaveAlternateScreen);
            let _ = disable_raw_mode();
            return Err(error);
        }
        // Mouse capture is best-effort; restricted terminals retain keyboard navigation.
        let _ = execute!(stdout, crossterm::event::EnableMouseCapture);
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        // Best-effort restoration also runs while unwinding from a panic.
        let mut stdout = io::stdout();
        let _ = execute!(stdout, crossterm::event::DisableMouseCapture);
        let _ = execute!(stdout, Show, LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InputMode {
    Events,
    Lines,
}

const RUSTY_REALM_ASCII_ART: &str = r#"      ▄▄▄▄▄                ▄▓▄▄          ▄▄▄▄▄          ▄▄▀▀▀████░░▒▓▄       ▄▄     ▄        ▄▄▄▄▄        ▄▄▀▀▀████░░▒▓▄     ▄▄▄▄▄▄▄          ▄                ▄▄▄▄▄▄▄       
  ▄▄███░░▒▒▓▓▄       ▄█▀   ▐▒▓▓▄     ▄▄███░░▒▒▓▄▌      ▀       ▄ ███▒▓▌   ▄▓▓▒▒░   ▓▓█   ▄▄███░░▒▒▓▓▄    ▀  ▄     ▀███▒▓▌  ▄█▀▀▀█░░▒▓▓▄      █▓▓             ▄█▀▀▀█░░▒▓▓▄    
▄▀▀ ▄   ▀ ░▒▓▓▌    ▄▒▓      █▒▓▓▄  ▄▀▀ ▄   ▀ ░▒▓▌             ▐▒▓ ▀██▒▓  ▐▓▓▒░ ▀   ▓░▒ ▄▀▀ ▄   ▀ ░▒▓▓▌     ▐▒▓      ▀██▒▓ ▀▄  ▄   ▀ ░▒▓▓▄    ▒░▓            ▀▄  ▄   █ ░▒▓▓▄  
 ▀ ▐▓▓   ▐█░▒▓    ▐░░▌      ▐░░▒▓   ▀ ▐▓▓    ▀░▀              ▐░░   ░     ▓▒░█▌    ░░▌  ▀ ▐▓▓   ▐█░▒▓      ▐░░        ░      ▐▒▓    ▀█░▒▓▌   ▐░░               ▐▒▓  ▐███░▒▓▌ 
   ▐▒░  ▄█░▒▀     ▐░░        ██░▒▌    ▐▒░    ▄▄▄▄▄            ▐░░  ▀       ▀▒░█▄   ░░▌    ▐▒░  ▄█░▒▀       ▐░░  ▄    ▀       ▐░░  ▄   ▀█░░   ▐░░               ▐░░   █▀ ▀█░░ 
    ░▌▄▀▀▀▄▄▄▄     ▒█        ███░▌     ▀  ▄██░▒▓▓▒░▄           ▒▌            ▀▀▀█▄ ▐▒      ░▌▄▀▀▀▄▄▄▄       ▒▌ ▐▒▌           ▐░░ ▐▒▌   ▐█░▌   ▒▌    ▀▄▄        ▐░░   █   ▐█░▌
    ▓▌   ▀▀█░▒▓▄   ▐▓▌      ▐██░█▌      ▄█▀▀   ▀▀░▒▓▄          ▓▌         ▄▄▄    ▀▀▐▓      ▓▌   ▀▀█░▒▓▄     ▓▌  ▀    ▀▄       ▒▌  ▀      █    ▓▌      ▀▒▓▄      ▒▌   ▒     █ 
    █      ▐█░▒▓▌   ▀█▄     ██░██                ▐░▒▓▌         █        ▐▒░█▌      ▐█      █      ▐█░▒▓▌    █         ▒▓▄     ▓▌        ▐▌    █       ▐░▒▓▌     ▓▌   ░    ▐▌ 
    ▓      ▐█░▒▓▌  ▄  ▀█▀ ▄████       ▀▄        ▄█░▒▓          ▐        ▓▒░██      ▀       ▓      ▐█░▒▓▌    ▐       ▄█░▒▓     █         █   ▄ ▐     ▄▄██░▒▌     █         █  
    ▐      █░▒▓░    ▀█▄▄▄████▀▀         ▀█▄▄▄▄███░▒▀                    ░▓▒░██▄▄▄▄█▀       ▐      █░▒▓░   ▀▄▄   ▄▄▄█░█░░▀     ▐        ▄▀    █▄▄▄▄████ ░█▀      ▐        ▄▀    
            ▀▀▀       ▀▀▀▀▀                ▀▀▀▀▀▀                        ▀▀▀▀▀▀▀▀▀                 ▀▀▀       ▀▀▀▀▀▀▀▀▀                ▀         ▀▀▀▀▀▀▀▀                ▀"#;

const STARTUP_LOGO_WIDTH: u16 = 175;
const STARTUP_LOGO_HEIGHT: u16 = 11;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccountLandingChoice {
    Account(usize),
    Action(usize),
}

fn startup_logo_fits(area: Rect) -> bool {
    area.width >= STARTUP_LOGO_WIDTH + 6 && area.height >= 26
}

pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    _guard: Option<TerminalGuard>,
    input_mode: InputMode,
}

impl Tui {
    fn select_menu_lines(&self, title: &str, options: &[&str]) -> io::Result<usize> {
        loop {
            println!("\n{title}");
            for (index, option) in options.iter().enumerate() {
                let (kind, label, reason) = menu_option_parts(option);
                let line = match kind {
                    MenuOptionKind::Actionable => label.to_string(),
                    MenuOptionKind::Disabled => format!(
                        "[DISABLED] {label} — {}",
                        reason.unwrap_or("This option is disabled.")
                    ),
                    MenuOptionKind::Unavailable => format!(
                        "[UNAVAILABLE] {label} — {}",
                        reason.unwrap_or("This option is unavailable.")
                    ),
                };
                println!("  {}. {}", index + 1, line);
            }
            let prompt = format!(
                "Type an option number or exact label (1-{}), then press Enter: ",
                options.len()
            );
            let raw = read_line_prompt(&prompt)?;
            let choice = raw.trim();
            let selected = choice
                .parse::<usize>()
                .ok()
                .filter(|number| (1..=options.len()).contains(number))
                .map(|number| number - 1)
                .or_else(|| {
                    options.iter().position(|option| {
                        let (_, label, _) = menu_option_parts(option);
                        label.eq_ignore_ascii_case(choice)
                    })
                });
            if let Some(index) = selected {
                let (kind, label, reason) = menu_option_parts(options[index]);
                if kind != MenuOptionKind::Actionable {
                    println!(
                        "{} cannot be selected: {}",
                        label,
                        reason.unwrap_or("This option is not currently available.")
                    );
                    continue;
                }
                return Ok(index);
            }
            println!("Invalid selection. Enter a listed number or exact option label.");
        }
    }

    fn switch_to_line_input(&mut self) {
        self.input_mode = InputMode::Lines;
        drop(self._guard.take());
    }

    pub fn new() -> io::Result<Self> {
        let configured_mode = std::env::var("RUSTYREALM_INPUT_MODE").ok();
        let use_lines =
            should_use_line_input(configured_mode.as_deref(), io::stdin().is_terminal());
        let (guard, input_mode) = if use_lines {
            (None, InputMode::Lines)
        } else {
            match TerminalGuard::new() {
                Ok(guard) => (Some(guard), InputMode::Events),
                // Some terminal wrappers do not support raw-mode event polling.
                // Start in line-input mode instead of failing application startup.
                Err(_) => (None, InputMode::Lines),
            }
        };
        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = Terminal::new(backend)?;
        if input_mode == InputMode::Events {
            terminal.clear()?;
        }
        Ok(Self {
            terminal,
            _guard: guard,
            input_mode,
        })
    }

    /// Two-panel local account landing screen. The left panel lists saved
    /// accounts and the right panel contains account-management actions.
    pub fn select_account_landing(
        &mut self,
        title: &str,
        accounts: &[String],
        actions: &[&str],
    ) -> io::Result<AccountLandingChoice> {
        if self.input_mode == InputMode::Lines {
            let mut labels: Vec<String> = accounts
                .iter()
                .map(|account| format!("Account: {account}"))
                .collect();
            labels.extend(actions.iter().map(|action| (*action).to_string()));
            let refs: Vec<&str> = labels.iter().map(String::as_str).collect();
            let choice = self.select_menu_lines(title, &refs)?;
            return Ok(if choice < accounts.len() {
                AccountLandingChoice::Account(choice)
            } else {
                AccountLandingChoice::Action(choice - accounts.len())
            });
        }
        if actions.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "the account landing screen requires at least one action",
            ));
        }

        let mut selected_account = 0usize;
        let mut selected_action = 0usize;
        let mut focus_right = accounts.is_empty();
        loop {
            let mut account_hit_area = Rect::default();
            let mut action_hit_area = Rect::default();
            let mut account_offset = 0usize;
            let mut action_offset = 0usize;

            self.render_screen(None, |frame, area, _| {
                let account_width = accounts
                    .iter()
                    .map(|account| account.chars().count())
                    .max()
                    .unwrap_or(20)
                    .saturating_add(8)
                    .clamp(28, 64)
                    .min(usize::from(area.width)) as u16;
                let action_width = actions
                    .iter()
                    .map(|action| action.chars().count())
                    .max()
                    .unwrap_or(16)
                    .saturating_add(8)
                    .clamp(28, 64)
                    .min(usize::from(area.width)) as u16;
                let side_by_side =
                    area.width >= account_width.saturating_add(action_width).saturating_add(4);
                let show_logo = startup_logo_fits(area);
                let logo_height = if show_logo { STARTUP_LOGO_HEIGHT } else { 0 };
                let title_height = if area.height >= 7 { 3 } else { 1 };
                let footer_height = if area.height >= 8 { 2 } else { 0 };
                let account_natural_height = accounts.len().saturating_add(2).clamp(4, 12) as u16;
                let action_natural_height = actions.len().saturating_add(2).clamp(4, 12) as u16;
                let chrome_height = logo_height
                    .saturating_add(if show_logo { 1 } else { 0 })
                    .saturating_add(title_height)
                    .saturating_add(1)
                    .saturating_add(footer_height);
                let panel_budget = area.height.saturating_sub(chrome_height).max(1);
                let (account_height, action_height) = if side_by_side {
                    let height = account_natural_height
                        .max(action_natural_height)
                        .min(panel_budget);
                    (height, height)
                } else {
                    let each = panel_budget.saturating_sub(1) / 2;
                    (
                        account_natural_height.min(each.max(1)),
                        action_natural_height.min(each.max(1)),
                    )
                };
                let panels_height = if side_by_side {
                    account_height
                } else {
                    account_height
                        .saturating_add(1)
                        .saturating_add(action_height)
                };
                let total_height = logo_height
                    .saturating_add(if show_logo { 1 } else { 0 })
                    .saturating_add(title_height)
                    .saturating_add(1)
                    .saturating_add(panels_height)
                    .saturating_add(footer_height);
                let top = area
                    .y
                    .saturating_add(area.height.saturating_sub(total_height) / 2);
                let mut y = top;

                if show_logo {
                    let logo_area = Rect::new(area.x, y, area.width, logo_height);
                    frame.render_widget(
                        Paragraph::new(RUSTY_REALM_ASCII_ART)
                            .alignment(Alignment::Center)
                            .style(Style::default().fg(Color::Gray)),
                        logo_area,
                    );
                    y = y.saturating_add(logo_height).saturating_add(1);
                }

                let title_area = Rect::new(area.x, y, area.width, title_height.min(area.height));
                let heading = Paragraph::new(title)
                    .alignment(Alignment::Center)
                    .style(
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )
                    .block(Block::default().borders(Borders::ALL));
                frame.render_widget(heading, title_area);
                y = y.saturating_add(title_height).saturating_add(1);

                if side_by_side {
                    let combined_width = account_width
                        .saturating_add(action_width)
                        .saturating_add(4)
                        .min(area.width);
                    let left_x = area
                        .x
                        .saturating_add(area.width.saturating_sub(combined_width) / 2);
                    let account_rect = Rect::new(left_x, y, account_width, account_height);
                    let action_rect = Rect::new(
                        left_x.saturating_add(account_width).saturating_add(4),
                        y,
                        action_width,
                        action_height,
                    );
                    render_account_panel(
                        frame,
                        account_rect,
                        "Active Accounts",
                        accounts,
                        selected_account,
                        !focus_right,
                        &mut account_hit_area,
                        &mut account_offset,
                    );
                    render_action_panel(
                        frame,
                        action_rect,
                        "Options",
                        actions,
                        selected_action,
                        focus_right,
                        &mut action_hit_area,
                        &mut action_offset,
                    );
                } else {
                    let panel_width = account_width.max(action_width).min(area.width);
                    let left_x = area
                        .x
                        .saturating_add(area.width.saturating_sub(panel_width) / 2);
                    let account_rect = Rect::new(left_x, y, panel_width, account_height);
                    render_account_panel(
                        frame,
                        account_rect,
                        "Active Accounts",
                        accounts,
                        selected_account,
                        !focus_right,
                        &mut account_hit_area,
                        &mut account_offset,
                    );
                    y = y.saturating_add(account_height).saturating_add(1);
                    let action_rect = Rect::new(left_x, y, panel_width, action_height);
                    render_action_panel(
                        frame,
                        action_rect,
                        "Options",
                        actions,
                        selected_action,
                        focus_right,
                        &mut action_hit_area,
                        &mut action_offset,
                    );
                }

                if footer_height > 0 {
                    let footer = Paragraph::new(
                        "Tab / ← / →: switch panel  ·  ↑ / ↓: select  ·  Enter: open  ·  Esc: exit",
                    )
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::DarkGray))
                    .block(Block::default().borders(Borders::ALL));
                    let footer_y = top
                        .saturating_add(total_height)
                        .saturating_sub(footer_height);
                    frame.render_widget(
                        footer,
                        Rect::new(area.x, footer_y, area.width, footer_height),
                    );
                }
            })?;

            let input_event = match event::read() {
                Ok(event) => event,
                Err(_) if self.input_mode == InputMode::Events => {
                    self.switch_to_line_input();
                    return self.select_account_landing(title, accounts, actions);
                }
                Err(error) => return Err(error),
            };
            match input_event {
                Event::Resize(_, _) => continue,
                Event::Mouse(mouse) if matches!(mouse.kind, MouseEventKind::Moved) => {
                    if !accounts.is_empty() {
                        if let Some(index) = menu_row_at(
                            account_hit_area,
                            accounts.len(),
                            account_offset,
                            mouse.column,
                            mouse.row,
                        ) {
                            selected_account = index;
                            focus_right = false;
                        }
                    }
                    if let Some(index) = menu_row_at(
                        action_hit_area,
                        actions.len(),
                        action_offset,
                        mouse.column,
                        mouse.row,
                    ) {
                        selected_action = index;
                        focus_right = true;
                    }
                }
                Event::Mouse(mouse)
                    if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) =>
                {
                    if !accounts.is_empty() {
                        if let Some(index) = menu_row_at(
                            account_hit_area,
                            accounts.len(),
                            account_offset,
                            mouse.column,
                            mouse.row,
                        ) {
                            selected_account = index;
                            return Ok(AccountLandingChoice::Account(index));
                        }
                    }
                    if let Some(index) = menu_row_at(
                        action_hit_area,
                        actions.len(),
                        action_offset,
                        mouse.column,
                        mouse.row,
                    ) {
                        selected_action = index;
                        return Ok(AccountLandingChoice::Action(index));
                    }
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Tab | KeyCode::Left | KeyCode::Right,
                    ..
                }) => {
                    focus_right = if accounts.is_empty() {
                        true
                    } else {
                        !focus_right
                    };
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Up, ..
                })
                | Event::Key(KeyEvent {
                    code: KeyCode::Char('k'),
                    ..
                }) => {
                    if focus_right {
                        selected_action = if selected_action == 0 {
                            actions.len() - 1
                        } else {
                            selected_action - 1
                        };
                    } else if !accounts.is_empty() {
                        selected_account = if selected_account == 0 {
                            accounts.len() - 1
                        } else {
                            selected_account - 1
                        };
                    }
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Down,
                    ..
                })
                | Event::Key(KeyEvent {
                    code: KeyCode::Char('j'),
                    ..
                }) => {
                    if focus_right {
                        selected_action = (selected_action + 1) % actions.len();
                    } else if !accounts.is_empty() {
                        selected_account = (selected_account + 1) % accounts.len();
                    }
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    ..
                }) => {
                    return Ok(if focus_right {
                        AccountLandingChoice::Action(selected_action)
                    } else {
                        AccountLandingChoice::Account(selected_account)
                    });
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Esc, ..
                }) => return Ok(AccountLandingChoice::Action(actions.len() - 1)),
                _ => {}
            }
        }
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

    /// Keyboard- and mouse-enabled menu. Hit regions are recalculated on every draw.
    pub fn select_menu(
        &mut self,
        title: &str,
        options: &[&str],
        state: Option<&GameState>,
    ) -> io::Result<usize> {
        if options.is_empty() {
            return Ok(0);
        }
        if self.input_mode == InputMode::Lines {
            return self.select_menu_lines(title, options);
        }

        let mut selected = 0usize;
        loop {
            let mut menu_region = Rect::default();
            let mut menu_offset = 0usize;
            self.render_screen(state, |frame, area, game_state| {
                // Menus should follow their contents instead of stretching across
                // the full terminal. Long lists still receive a scrollable viewport.
                let title_height = if area.height >= 7 { 3 } else { 0 };
                let footer_height = if area.height >= 10 { 2 } else { 0 };
                let menu_height = options.len().saturating_add(2).min(12).min(usize::from(
                    area.height.saturating_sub(title_height + footer_height),
                )) as u16;
                let content_height = area.height.saturating_sub(title_height + footer_height);
                let menu_width = options
                    .iter()
                    .map(|option| menu_option_parts(option).1.chars().count())
                    .max()
                    .unwrap_or(12)
                    .saturating_add(8)
                    .clamp(24, 64)
                    .min(usize::from(area.width)) as u16;
                let mut constraints = Vec::new();
                if title_height > 0 {
                    constraints.push(Constraint::Length(title_height));
                }
                constraints.push(Constraint::Length(menu_height.min(content_height)));
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
                        .style(
                            Style::default()
                                .fg(theme_color(game_state))
                                .add_modifier(Modifier::BOLD),
                        )
                        .block(Block::default().borders(Borders::ALL));
                    frame.render_widget(header, chunks[index]);
                    index += 1;
                }

                let menu_area = centered_rect_absolute(menu_width, menu_height, chunks[index]);
                let visible_options: Vec<ListItem<'_>> = options
                    .iter()
                    .enumerate()
                    .map(|(i, option)| {
                        let prefix = if i < 9 {
                            format!("{}) ", i + 1)
                        } else {
                            "   ".to_string()
                        };
                        let (kind, label, reason) = menu_option_parts(option);
                        let line = match kind {
                            MenuOptionKind::Disabled => {
                                let style = Style::default().fg(Color::DarkGray);
                                Line::from(vec![
                                    Span::styled(prefix, style),
                                    Span::styled("× ", style.add_modifier(Modifier::BOLD)),
                                    Span::styled(label.to_string(), style),
                                    Span::styled(
                                        format!(" — {}", reason.unwrap_or("Disabled")),
                                        style,
                                    ),
                                ])
                            }
                            MenuOptionKind::Unavailable => {
                                let style = Style::default().fg(Color::Yellow);
                                Line::from(vec![
                                    Span::styled(prefix, style),
                                    Span::styled("! ", style.add_modifier(Modifier::BOLD)),
                                    Span::styled(label.to_string(), style),
                                    Span::styled(
                                        format!(" — {}", reason.unwrap_or("Unavailable")),
                                        style,
                                    ),
                                ])
                            }
                            MenuOptionKind::Actionable => {
                                let prefix_span = Span::styled(
                                    prefix,
                                    Style::default()
                                        .fg(Color::Green)
                                        .add_modifier(Modifier::BOLD),
                                );
                                if let Some(rest) = label.strip_prefix('[') {
                                    if let Some((rarity, name)) = rest.split_once("] ") {
                                        let color = match rarity {
                                            "Common" => Color::Gray,
                                            "Uncommon" => Color::Green,
                                            "Rare" => Color::Blue,
                                            "Epic" => Color::Magenta,
                                            "Legendary" => Color::Yellow,
                                            _ => Color::White,
                                        };
                                        Line::from(vec![
                                            prefix_span,
                                            Span::styled(
                                                format!("[{rarity}] "),
                                                Style::default()
                                                    .fg(color)
                                                    .add_modifier(Modifier::BOLD),
                                            ),
                                            Span::raw(name.to_string()),
                                        ])
                                    } else {
                                        Line::from(vec![prefix_span, Span::raw(label.to_string())])
                                    }
                                } else {
                                    Line::from(vec![prefix_span, Span::raw(label.to_string())])
                                }
                            }
                        };
                        ListItem::new(line)
                    })
                    .collect();

                let list = List::new(visible_options)
                    .block(
                        Block::default()
                            .title(if title_height == 0 {
                                title
                            } else {
                                " Options "
                            })
                            .borders(Borders::ALL),
                    )
                    .highlight_style(
                        Style::default()
                            .fg(Color::Black)
                            .bg(theme_color(game_state))
                            .add_modifier(Modifier::BOLD),
                    )
                    .highlight_symbol("▶ ");
                let mut list_state = ListState::default();
                list_state.select(Some(selected.min(options.len().saturating_sub(1))));
                menu_region = Block::default()
                    .title(if title_height == 0 {
                        title
                    } else {
                        " Options "
                    })
                    .borders(Borders::ALL)
                    .inner(menu_area);
                frame.render_stateful_widget(list, menu_area, &mut list_state);
                menu_offset = list_state.offset();
                index += 1;

                if footer_height > 0 {
                    let footer = Paragraph::new("↑/↓ or j/k: move  ·  Enter: select  ·  Esc: back")
                        .alignment(Alignment::Center)
                        .style(Style::default().fg(Color::DarkGray))
                        .block(Block::default().borders(Borders::ALL));
                    frame.render_widget(footer, chunks[index]);
                }
            })?;

            let input_event = match event::read() {
                Ok(event) => event,
                Err(_) if self.input_mode == InputMode::Events => {
                    self.switch_to_line_input();
                    return self.select_menu_lines(title, options);
                }
                Err(error) => return Err(error),
            };
            match input_event {
                Event::Mouse(mouse)
                    if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) =>
                {
                    if let Some(index) = menu_row_at(
                        menu_region,
                        options.len(),
                        menu_offset,
                        mouse.column,
                        mouse.row,
                    ) {
                        selected = index;
                        if let Some((label, reason)) = menu_option_reason(options[index]) {
                            self.show_message(
                                "Option unavailable",
                                &format!("{label}\n\n{reason}"),
                                state,
                            )?;
                        } else {
                            return Ok(index);
                        }
                    }
                }
                Event::Mouse(mouse) if matches!(mouse.kind, MouseEventKind::Moved) => {
                    if let Some(index) = menu_row_at(
                        menu_region,
                        options.len(),
                        menu_offset,
                        mouse.column,
                        mouse.row,
                    ) {
                        selected = index;
                    }
                }
                Event::Resize(_, _) => continue,
                Event::Key(KeyEvent {
                    code: KeyCode::Up, ..
                })
                | Event::Key(KeyEvent {
                    code: KeyCode::Char('k'),
                    ..
                }) => {
                    selected = if selected == 0 {
                        options.len() - 1
                    } else {
                        selected - 1
                    };
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Down,
                    ..
                })
                | Event::Key(KeyEvent {
                    code: KeyCode::Char('j'),
                    ..
                }) => {
                    selected = if selected + 1 >= options.len() {
                        0
                    } else {
                        selected + 1
                    };
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    ..
                }) => {
                    if let Some((label, reason)) = menu_option_reason(options[selected]) {
                        self.show_message(
                            "Option unavailable",
                            &format!("{label}\n\n{reason}"),
                            state,
                        )?;
                    } else {
                        return Ok(selected);
                    }
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Esc, ..
                }) => {
                    let back_index = options
                        .iter()
                        .position(|option| {
                            let label = option.to_ascii_lowercase();
                            label.contains("back")
                                || label.contains("exit")
                                || label.contains("leave")
                                || label.contains("return")
                                || label.contains("flee")
                                || label.contains("cancel")
                        })
                        .unwrap_or(options.len() - 1);
                    return Ok(back_index);
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char(c),
                    ..
                }) if c.is_ascii_digit() => {
                    let digit = c.to_digit(10).unwrap_or(0) as usize;
                    if digit > 0 && digit <= options.len() && digit <= 9 {
                        let index = digit - 1;
                        selected = index;
                        if let Some((label, reason)) = menu_option_reason(options[index]) {
                            self.show_message(
                                "Option unavailable",
                                &format!("{label}\n\n{reason}"),
                                state,
                            )?;
                        } else {
                            return Ok(index);
                        }
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
        if self.input_mode == InputMode::Lines {
            if masked {
                println!("Note: line-input mode cannot hide typed characters; input will be echoed by the terminal.");
            }
            let input = read_line_prompt(&format!("{prompt}: "))?;
            return Ok(if masked {
                input
            } else {
                input.trim().to_string()
            });
        }
        let mut input = String::new();
        loop {
            self.render_screen(state, |frame, area, game_state| {
                let popup_width = prompt.chars().count().saturating_add(12).clamp(32, 72);
                let popup = centered_rect_absolute(
                    popup_width.min(usize::from(area.width)) as u16,
                    5.min(area.height),
                    area,
                );
                frame.render_widget(Clear, popup);
                let block = Block::default()
                    .title(format!(" {} ", prompt))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme_color(game_state)));
                let inner = block.inner(popup);
                frame.render_widget(block, popup);

                let shown = if masked {
                    "*".repeat(input.chars().count())
                } else {
                    input.clone()
                };
                let paragraph = Paragraph::new(format!("> {}", shown))
                    .style(Style::default().fg(Color::White))
                    .wrap(Wrap { trim: true });
                frame.render_widget(paragraph, inner);

                if inner.width > 0 && inner.height > 0 {
                    let input_cells = input.chars().count();
                    let cursor_x = inner.x.saturating_add(2).saturating_add(
                        input_cells.min(inner.width.saturating_sub(3) as usize) as u16,
                    );
                    frame.set_cursor(cursor_x.min(inner.x + inner.width - 1), inner.y);
                }
            })?;

            let input_event = match event::read() {
                Ok(event) => event,
                Err(_) if self.input_mode == InputMode::Events => {
                    self.switch_to_line_input();
                    if masked {
                        println!("Note: line-input mode cannot hide typed characters; input will be echoed by the terminal.");
                    }
                    let input = read_line_prompt(&format!("{prompt}: "))?;
                    return Ok(if masked {
                        input
                    } else {
                        input.trim().to_string()
                    });
                }
                Err(error) => return Err(error),
            };
            match input_event {
                Event::Resize(_, _) => continue,
                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    ..
                }) => {
                    if masked {
                        return Ok(input);
                    }
                    return Ok(input.trim().to_string());
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Esc, ..
                }) => return Ok(String::new()),
                Event::Key(KeyEvent {
                    code: KeyCode::Char(c),
                    ..
                }) => input.push(c),
                Event::Key(KeyEvent {
                    code: KeyCode::Backspace,
                    ..
                }) => {
                    input.pop();
                }
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
        if self.input_mode == InputMode::Lines {
            println!("\n{title}\n\n{message}\n\nPress Enter to continue.");
            let _ = read_line_prompt("")?;
            return Ok(());
        }
        let display_message = format!("{}\n\nPress Enter or Esc to continue.", message);
        let mut scroll = 0u16;
        loop {
            self.render_screen(state, |frame, area, game_state| {
                let message_width = message
                    .lines()
                    .map(|line| line.chars().count())
                    .max()
                    .unwrap_or(24)
                    .saturating_add(6)
                    .clamp(40, 88);
                let wrapped_lines = message
                    .lines()
                    .map(|line| {
                        let width = message_width.saturating_sub(4).max(1);
                        line.chars().count().div_ceil(width)
                    })
                    .sum::<usize>()
                    .max(1);
                let message_height = wrapped_lines.saturating_add(6).clamp(8, 24);
                let popup = centered_rect_absolute(
                    message_width.min(usize::from(area.width)) as u16,
                    message_height.min(usize::from(area.height)) as u16,
                    area,
                );
                frame.render_widget(Clear, popup);
                let block = Block::default()
                    .title(format!(" {} ", title))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme_color(game_state)));
                let inner = block.inner(popup);
                frame.render_widget(block, popup);
                let paragraph = Paragraph::new(display_message.as_str())
                    .wrap(Wrap { trim: true })
                    .scroll((scroll, 0));
                frame.render_widget(paragraph, inner);
            })?;

            let input_event = match event::read() {
                Ok(event) => event,
                Err(_) if self.input_mode == InputMode::Events => {
                    self.switch_to_line_input();
                    println!("\n{title}\n\n{message}\n\nPress Enter to continue.");
                    let _ = read_line_prompt("")?;
                    return Ok(());
                }
                Err(error) => return Err(error),
            };
            match input_event {
                Event::Resize(_, _) => continue,
                Event::Key(KeyEvent {
                    code: KeyCode::Up, ..
                }) => scroll = scroll.saturating_sub(1),
                Event::Key(KeyEvent {
                    code: KeyCode::Down,
                    ..
                }) => scroll = scroll.saturating_add(1),
                Event::Key(KeyEvent {
                    code: KeyCode::PageUp,
                    ..
                }) => scroll = scroll.saturating_sub(8),
                Event::Key(KeyEvent {
                    code: KeyCode::PageDown,
                    ..
                }) => scroll = scroll.saturating_add(8),
                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    ..
                })
                | Event::Key(KeyEvent {
                    code: KeyCode::Esc, ..
                })
                | Event::Key(KeyEvent {
                    code: KeyCode::Char(' '),
                    ..
                }) => return Ok(()),
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
        if self.input_mode == InputMode::Lines {
            println!("\n{title}\n\n{content}\n\nPress Enter to return.");
            let _ = read_line_prompt("")?;
            return Ok(());
        }
        let mut scroll = 0u16;
        loop {
            self.render_screen(state, |frame, area, game_state| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Min(1),
                        Constraint::Length(if area.height >= 5 { 2 } else { 0 }),
                    ])
                    .split(area);
                let header = Paragraph::new(title)
                    .alignment(Alignment::Center)
                    .style(
                        Style::default()
                            .fg(theme_color(game_state))
                            .add_modifier(Modifier::BOLD),
                    )
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

            let input_event = match event::read() {
                Ok(event) => event,
                Err(_) if self.input_mode == InputMode::Events => {
                    self.switch_to_line_input();
                    println!("\n{title}\n\n{content}\n\nPress Enter to return.");
                    let _ = read_line_prompt("")?;
                    return Ok(());
                }
                Err(error) => return Err(error),
            };
            match input_event {
                Event::Resize(_, _) => continue,
                Event::Key(KeyEvent {
                    code: KeyCode::Up, ..
                }) => scroll = scroll.saturating_sub(1),
                Event::Key(KeyEvent {
                    code: KeyCode::Down,
                    ..
                }) => scroll = scroll.saturating_add(1),
                Event::Key(KeyEvent {
                    code: KeyCode::PageUp,
                    ..
                }) => scroll = scroll.saturating_sub(8),
                Event::Key(KeyEvent {
                    code: KeyCode::PageDown,
                    ..
                }) => scroll = scroll.saturating_add(8),
                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    ..
                })
                | Event::Key(KeyEvent {
                    code: KeyCode::Esc, ..
                }) => return Ok(()),
                _ => {}
            }
        }
    }
}

fn render_account_panel(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &str,
    accounts: &[String],
    selected: usize,
    focused: bool,
    hit_area: &mut Rect,
    offset: &mut usize,
) {
    let block = Block::default()
        .title(format!(" {title} "))
        .borders(Borders::ALL);
    *hit_area = block.inner(area);
    if accounts.is_empty() {
        frame.render_widget(block, area);
        frame.render_widget(
            Paragraph::new("No local accounts yet")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray)),
            *hit_area,
        );
        *offset = 0;
        return;
    }
    let items: Vec<ListItem<'_>> = accounts
        .iter()
        .map(|account| ListItem::new(Line::from(account.as_str())))
        .collect();
    let mut state = ListState::default();
    state.select(Some(selected.min(accounts.len().saturating_sub(1))));
    let list = List::new(items)
        .block(block)
        .highlight_style(if focused {
            Style::default()
                .fg(Color::Black)
                .bg(theme_color(None))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(theme_color(None))
                .add_modifier(Modifier::BOLD)
        })
        .highlight_symbol(if focused { "▶ " } else { "  " });
    frame.render_stateful_widget(list, area, &mut state);
    *offset = state.offset();
}

fn render_action_panel(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &str,
    actions: &[&str],
    selected: usize,
    focused: bool,
    hit_area: &mut Rect,
    offset: &mut usize,
) {
    let block = Block::default()
        .title(format!(" {title} "))
        .borders(Borders::ALL);
    *hit_area = block.inner(area);
    let items: Vec<ListItem<'_>> = actions
        .iter()
        .map(|action| ListItem::new(Line::from(*action)))
        .collect();
    let mut state = ListState::default();
    state.select(Some(selected.min(actions.len().saturating_sub(1))));
    let list = List::new(items)
        .block(block)
        .highlight_style(if focused {
            Style::default()
                .fg(Color::Black)
                .bg(theme_color(None))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(theme_color(None))
                .add_modifier(Modifier::BOLD)
        })
        .highlight_symbol(if focused { "▶ " } else { "  " });
    frame.render_stateful_widget(list, area, &mut state);
    *offset = state.offset();
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MenuOptionKind {
    Actionable,
    Disabled,
    Unavailable,
}

fn menu_option_parts(option: &str) -> (MenuOptionKind, &str, Option<&str>) {
    if let Some(rest) = option.strip_prefix("[Disabled: ") {
        if let Some((reason, label)) = rest.split_once("] ") {
            return (MenuOptionKind::Disabled, label, Some(reason));
        }
    }
    if let Some(rest) = option.strip_prefix("[Unavailable: ") {
        if let Some((reason, label)) = rest.split_once("] ") {
            return (MenuOptionKind::Unavailable, label, Some(reason));
        }
    }
    (MenuOptionKind::Actionable, option, None)
}

fn menu_option_reason(option: &str) -> Option<(&str, &str)> {
    let (kind, label, reason) = menu_option_parts(option);
    if kind == MenuOptionKind::Actionable {
        None
    } else {
        Some((
            label,
            reason.unwrap_or("This option is not currently available."),
        ))
    }
}

fn should_use_line_input(configured_mode: Option<&str>, stdin_is_terminal: bool) -> bool {
    let configured_mode = configured_mode
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    matches!(
        configured_mode.as_str(),
        "text" | "line" | "plain" | "stdin"
    ) || !stdin_is_terminal
}

fn read_line_prompt(prompt: &str) -> io::Result<String> {
    let mut stdout = io::stdout();
    write!(stdout, "{prompt}")?;
    stdout.flush()?;

    let mut input = String::new();
    if io::stdin().read_line(&mut input)? == 0 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "text input stream closed before a line was received",
        ));
    }
    Ok(input.trim_end_matches(&['\r', '\n'][..]).to_string())
}

/// Maps an absolute terminal coordinate to the currently rendered menu row.
fn menu_row_at(
    area: Rect,
    option_count: usize,
    first_visible: usize,
    x: u16,
    y: u16,
) -> Option<usize> {
    if option_count == 0
        || x < area.x
        || x >= area.x.saturating_add(area.width)
        || y < area.y
        || y >= area.y.saturating_add(area.height)
    {
        return None;
    }
    let row = first_visible.saturating_add(usize::from(y.saturating_sub(area.y)));
    (row < option_count && row < first_visible.saturating_add(usize::from(area.height)))
        .then_some(row)
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
        .border_style(Style::default().fg(theme_color(Some(state))));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(inner);

    frame.render_widget(
        Paragraph::new(format!(
            "{}{} · Combat {}",
            state.player.username,
            state
                .active_title
                .as_deref()
                .map(|t| format!(" · {t}"))
                .unwrap_or_default(),
            state.player.combat_level
        ))
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .wrap(Wrap { trim: true }),
        chunks[0],
    );

    let hp_ratio = if state.player.max_hp > 0 {
        (state.player.hp as f64 / state.player.max_hp as f64).clamp(0.0, 1.0)
    } else {
        0.0
    };
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
        (1.0 - state.player.xp_until_next as f64 / state.player.orig_xp_threshold as f64)
            .clamp(0.0, 1.0)
    } else {
        0.0
    };
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
    let title = state
        .active_title
        .as_deref()
        .map(|t| format!(" · {t}"))
        .unwrap_or_default();
    let text = format!(
        " {}{}  ·  Lv {}  ·  HP {}/{}  ·  Gold {}  ·  {} ",
        state.player.username,
        title,
        state.player.combat_level,
        state.player.hp,
        state.player.max_hp,
        state.player.gold,
        state.player.weapon.name(),
    );
    frame.render_widget(
        Paragraph::new(text)
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(theme_color(Some(state)))
                    .bg(Color::DarkGray),
            )
            .block(Block::default().borders(Borders::ALL)),
        area,
    );
}

fn render_compact_header(frame: &mut Frame<'_>, area: Rect, state: &GameState) {
    let title = state
        .active_title
        .as_deref()
        .map(|t| format!(" · {t}"))
        .unwrap_or_default();
    let calendar = crate::calendar::calendar_at(state.world_turn);
    let season = calendar.season_name.chars().take(3).collect::<String>();
    let month = calendar.month_name.chars().take(3).collect::<String>();
    let date = format!(
        "Y{} {} {}{} {:02}:00",
        calendar.year, season, month, calendar.day, calendar.hour
    );
    let text = format!(
        "Lv{} HP:{}/{} St:{}/{} G:{} {}{}",
        state.player.combat_level,
        state.player.hp,
        state.player.max_hp,
        state.player.stamina,
        state.player.max_stamina,
        state.player.gold,
        date,
        title
    );
    frame.render_widget(
        Paragraph::new(text).style(Style::default().fg(theme_color(Some(state)))),
        area,
    );
}

fn centered_rect_absolute(width: u16, height: u16, area: Rect) -> Rect {
    let width = width.max(1).min(area.width);
    let height = height.max(1).min(area.height);
    Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    )
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
        .constraints([
            Constraint::Percentage(top),
            Constraint::Percentage(percent_y),
            Constraint::Percentage(bottom),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(left),
            Constraint::Percentage(percent_x),
            Constraint::Percentage(right),
        ])
        .split(vertical[1])[1]
}

pub fn theme_color(state: Option<&GameState>) -> Color {
    match state.map(|s| s.active_theme.as_str()).unwrap_or("Classic") {
        "Infernal Red" => Color::Red,
        "Golden Realm" => Color::Yellow,
        "Obsidian Shadow" => Color::DarkGray,
        _ => Color::Cyan,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;

    #[test]
    fn responsive_layout_renders_within_wide_standard_narrow_and_short_terminals() {
        use std::cell::Cell;

        let state = GameState::new(crate::state::Player::new("Hero".into(), String::new()));
        let cases = [
            (120, 30, 28, 0), // Wide: sidebar beside content.
            (90, 20, 28, 0),  // Exact wide breakpoint.
            (75, 25, 0, 3),   // Standard: compact header above content.
            (60, 20, 0, 3),   // Exact standard breakpoint.
            (45, 30, 0, 1),   // Narrow: minimal status row.
            (80, 12, 0, 1),   // Short: minimal status row regardless of width.
            (20, 4, 0, 0),    // Very short: reserve all space for content.
            (1, 1, 0, 0),
        ];

        for (width, height, expected_x, expected_y) in cases {
            let backend = TestBackend::new(width, height);
            let mut terminal = Terminal::new(backend).unwrap();
            let rendered = Cell::new(Rect::default());
            terminal
                .draw(|frame| {
                    let full = frame.size();
                    render_responsive_base(frame, full, Some(&state), |frame, content, _| {
                        rendered.set(content);
                        assert!(content.x.saturating_add(content.width) <= full.width);
                        assert!(content.y.saturating_add(content.height) <= full.height);
                        frame.render_widget(Paragraph::new("Responsive content"), content);
                    });
                })
                .unwrap();

            let content = rendered.get();
            assert_eq!(content.x, expected_x, "content x at {}x{}", width, height);
            assert_eq!(content.y, expected_y, "content y at {}x{}", width, height);
        }
    }
    #[test]
    fn original_startup_banner_is_exactly_sized_and_only_shown_on_large_terminals() {
        let max_width = RUSTY_REALM_ASCII_ART
            .lines()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or_default();
        assert_eq!(max_width, usize::from(STARTUP_LOGO_WIDTH));
        assert_eq!(
            RUSTY_REALM_ASCII_ART.lines().count(),
            usize::from(STARTUP_LOGO_HEIGHT)
        );
        assert!(startup_logo_fits(Rect::new(0, 0, 181, 26)));
        assert!(!startup_logo_fits(Rect::new(0, 0, 180, 40)));
        assert!(!startup_logo_fits(Rect::new(0, 0, 200, 25)));
    }

    #[test]
    fn line_input_mode_can_be_forced_and_is_selected_for_non_terminal_stdin() {
        assert!(should_use_line_input(Some("text"), true));
        assert!(should_use_line_input(Some("line"), true));
        assert!(should_use_line_input(Some("STDIN"), true));
        assert!(should_use_line_input(None, false));
        assert!(!should_use_line_input(None, true));
        assert!(!should_use_line_input(Some("auto"), true));
    }

    #[test]
    fn menu_option_statuses_preserve_reasons_and_only_block_non_actionable_entries() {
        assert_eq!(
            menu_option_parts("[Disabled: requires 10 stamina (have 3).] Shield Wall"),
            (
                MenuOptionKind::Disabled,
                "Shield Wall",
                Some("requires 10 stamina (have 3).")
            )
        );
        assert_eq!(
            menu_option_parts("[Unavailable: Out of range for this action.] Goblin · HP 8/20"),
            (
                MenuOptionKind::Unavailable,
                "Goblin · HP 8/20",
                Some("Out of range for this action.")
            )
        );
        assert_eq!(
            menu_option_parts("Basic attack"),
            (MenuOptionKind::Actionable, "Basic attack", None)
        );
        assert_eq!(menu_option_reason("Basic attack"), None);
        assert_eq!(
            menu_option_reason("[Disabled: no legal target.] Heavy Strike"),
            Some(("Heavy Strike", "no legal target."))
        );
    }

    #[test]
    fn menu_hit_testing_maps_only_visible_rows() {
        let area = Rect::new(4, 7, 20, 4);
        assert_eq!(menu_row_at(area, 3, 0, 4, 7), Some(0));
        assert_eq!(menu_row_at(area, 3, 0, 23, 9), Some(2));
        assert_eq!(menu_row_at(area, 3, 0, 3, 7), None);
        assert_eq!(menu_row_at(area, 3, 0, 24, 7), None);
        assert_eq!(menu_row_at(area, 3, 0, 10, 10), None);
        assert_eq!(menu_row_at(area, 3, 0, 10, 6), None);
        assert_eq!(menu_row_at(area, 0, 0, 10, 7), None);
        assert_eq!(menu_row_at(area, 10, 6, 10, 8), Some(7));
        assert_eq!(menu_row_at(area, 10, 6, 10, 10), Some(9));
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
