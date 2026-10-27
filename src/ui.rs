use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{Clear, ClearType},
    style::{Color, Print, ResetColor, SetForegroundColor},
    queue,
    cursor::MoveTo,
};
use std::io::{stdout, Write};

pub fn clear_screen() {
    let mut stdout = stdout();
    let _ = execute!(stdout, Clear(ClearType::All), MoveTo(0, 0));
}

pub fn print_boxed(title: &str, content: &[&str]) {
    let mut stdout = stdout();
    let width = 60;
    
    let _ = queue!(
        stdout,
        SetForegroundColor(Color::Cyan),
        Print(format!("┌{}┐\n", "─".repeat(width - 2))),
        Print(format!("│ {:<width$} │\n", title, width = width - 4)),
        Print(format!("├{}┤\n", "─".repeat(width - 2))),
        ResetColor
    );
    
    for line in content {
        let _ = queue!(
            stdout,
            Print(format!("│ {:<width$} │\n", line, width = width - 4))
        );
    }
    
    let _ = queue!(
        stdout,
        SetForegroundColor(Color::Cyan),
        Print(format!("└{}┘\n", "─".repeat(width - 2))),
        ResetColor
    );
    
    let _ = stdout.flush();
}

pub fn wait_for_enter() {
    let mut stdout = stdout();
    let _ = queue!(stdout, Print("\nPress Enter to continue... "));
    let _ = stdout.flush();
    
    loop {
        if let Ok(Event::Key(KeyEvent { code: KeyCode::Enter, .. })) = event::read() {
            break;
        }
    }
    clear_screen();
}

pub fn get_input(prompt: &str) -> String {
    let mut stdout = stdout();
    let _ = queue!(stdout, Print(prompt));
    let _ = stdout.flush();
    
    let mut input = String::new();
    loop {
        if let Ok(Event::Key(KeyEvent { code, .. })) = event::read() {
            match code {
                KeyCode::Enter => {
                    let _ = queue!(stdout, Print("\n"));
                    let _ = stdout.flush();
                    break;
                }
                KeyCode::Char(c) => {
                    input.push(c);
                    let _ = queue!(stdout, Print(c));
                    let _ = stdout.flush();
                }
                KeyCode::Backspace => {
                    if !input.is_empty() {
                        input.pop();
                        let _ = queue!(stdout, Print("\x08 \x08"));
                        let _ = stdout.flush();
                    }
                }
                _ => {}
            }
        }
    }
    input.trim().to_string()
}

pub fn get_hidden_input(prompt: &str) -> String {
    let mut stdout = stdout();
    let _ = queue!(stdout, Print(prompt));
    let _ = stdout.flush();
    
    let mut input = String::new();
    loop {
        if let Ok(Event::Key(KeyEvent { code, .. })) = event::read() {
            match code {
                KeyCode::Enter => {
                    let _ = queue!(stdout, Print("\n"));
                    let _ = stdout.flush();
                    break;
                }
                KeyCode::Char(c) => {
                    input.push(c);
                    let _ = queue!(stdout, Print("*"));
                    let _ = stdout.flush();
                }
                KeyCode::Backspace => {
                    if !input.is_empty() {
                        input.pop();
                        let _ = queue!(stdout, Print("\x08 \x08"));
                        let _ = stdout.flush();
                    }
                }
                _ => {}
            }
        }
    }
    input
}

pub fn print_centered(text: &str) {
    let mut stdout = stdout();
    let term_width = crossterm::terminal::size().unwrap_or((80, 24)).0 as usize;
    let padding = if text.len() < term_width {
        (term_width - text.len()) / 2
    } else {
        0
    };
    let _ = queue!(stdout, Print(format!("{}{}\n", " ".repeat(padding), text)));
    let _ = stdout.flush();
}

pub fn print_error(text: &str) {
    let mut stdout = stdout();
    let _ = queue!(
        stdout,
        SetForegroundColor(Color::Red),
        Print(format!("\n[ERROR] {}\n", text)),
        ResetColor
    );
    let _ = stdout.flush();
    wait_for_enter();
}

pub fn print_success(text: &str) {
    let mut stdout = stdout();
    let _ = queue!(
        stdout,
        SetForegroundColor(Color::Green),
        Print(format!("\n[SUCCESS] {}\n", text)),
        ResetColor
    );
    let _ = stdout.flush();
    wait_for_enter();
}
