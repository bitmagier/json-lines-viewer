use crate::model::{Message, Model};
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::prelude::Size;
use std::time::Duration;

pub fn handle_event(_: &Model) -> anyhow::Result<Option<Message>> {
    if event::poll(Duration::from_millis(250))? {
        match event::read()? {
            Event::Key(key) if key.kind == event::KeyEventKind::Press => {
                return Ok(handle_key(key));
            }
            Event::Resize(cols, rows) => {
                return Ok(handle_resize(cols, rows));
            }
            _ => (),
        }
    }
    Ok(None)
}

fn handle_key(key: event::KeyEvent) -> Option<Message> {
    match key.modifiers {
        KeyModifiers::NONE => match key.code {
            KeyCode::Home => Some(Message::First),
            KeyCode::End => Some(Message::Last),
            KeyCode::Up => Some(Message::ScrollUp),
            KeyCode::Down => Some(Message::ScrollDown),
            KeyCode::PageUp => Some(Message::PageUp),
            KeyCode::PageDown => Some(Message::PageDown),
            KeyCode::Left => Some(Message::ScrollLeft),
            KeyCode::Right => Some(Message::ScrollRight),
            KeyCode::Enter => Some(Message::Enter),
            KeyCode::Esc => Some(Message::Exit),
            KeyCode::Char('/') => Some(Message::OpenFindTask),
            KeyCode::Backspace => Some(Message::Backspace),
            KeyCode::Char(c) => Some(Message::CharacterInput(c)),
            _ => None,
        },
        KeyModifiers::SHIFT => match key.code {
            KeyCode::Char(c) => Some(Message::CharacterInput(c)),
            _ => None
        }
        KeyModifiers::CONTROL => match key.code {
            KeyCode::Char('s') => Some(Message::SaveSettings),
            KeyCode::Char('f') => Some(Message::OpenFindTask),
            _ => None,
        },
        _ => None,
    }
}

fn handle_resize(
    cols: u16,
    rows: u16,
) -> Option<Message> {
    Some(Message::Resized(Size { width: cols, height: rows }))
}
