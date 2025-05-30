use crate::model::{Message, Model};
use anyhow::Context;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::prelude::Size;
use std::time::Duration;

pub fn handle_event(_: &Model) -> anyhow::Result<Option<Message>> {
    let event_available = event::poll(Duration::from_millis(250)).context("failed to poll event")?;

    if !event_available {
        return Ok(None);
    }

    let event = event::read().context("failed to read event")?;
    let message = match event {
        Event::Key(key) if key.kind == event::KeyEventKind::Press => handle_key(key),
        Event::Resize(cols, rows) => handle_resize(cols, rows),
        _ => None,
    };

    Ok(message)
}

fn handle_key(key: event::KeyEvent) -> Option<Message> {
    Some(match key.modifiers {
        KeyModifiers::NONE => match key.code {
            KeyCode::Home => Message::First,
            KeyCode::End => Message::Last,
            KeyCode::Up => Message::ScrollUp,
            KeyCode::Down => Message::ScrollDown,
            KeyCode::PageUp => Message::PageUp,
            KeyCode::PageDown => Message::PageDown,
            KeyCode::Left => Message::ScrollLeft,
            KeyCode::Right => Message::ScrollRight,
            KeyCode::Enter => Message::Enter,
            KeyCode::Esc => Message::Exit,
            KeyCode::Char('/') => Message::OpenFindTask,
            KeyCode::Backspace => Message::Backspace,
            KeyCode::Char(c) => Message::CharacterInput(c),
            _ => return None,
        },
        KeyModifiers::SHIFT => match key.code {
            KeyCode::Char(c) => Message::CharacterInput(c),
            _ => return None,
        },
        KeyModifiers::CONTROL => match key.code {
            KeyCode::Char('s') => Message::SaveSettings,
            KeyCode::Char('f') => Message::OpenFindTask,
            _ => return None,
        },
        _ => return None,
    })
}

fn handle_resize(
    cols: u16,
    rows: u16,
) -> Option<Message> {
    Some(Message::Resized(Size { width: cols, height: rows }))
}
