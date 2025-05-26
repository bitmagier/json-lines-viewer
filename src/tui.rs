use crate::model::{Model, ModelViewState, Screen};
use ratatui::prelude::{Line, Style, Stylize, Text};
use ratatui::widgets::{Block, List, ListState};
use ratatui::{
    backend::{Backend, CrosstermBackend}, crossterm::{
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    Frame,
    Terminal,
};
use serde_json::Value;
use std::str::FromStr;
use std::{io::stdout, panic};

pub fn init_terminal() -> anyhow::Result<Terminal<impl Backend>> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    Ok(terminal)
}

pub fn restore_terminal() -> anyhow::Result<()> {
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

pub fn install_panic_hook() {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        stdout().execute(LeaveAlternateScreen).unwrap();
        disable_raw_mode().unwrap();
        original_hook(panic_info);
    }));
}

pub fn view(
    model: &mut Model,
    frame: &mut Frame,
) {
    let mut view_state: ModelViewState = model.view_state.clone();

    match model.active_screen {
        Screen::Done => (),
        Screen::Main => render_main_screen(model, &mut view_state.main_window_list_state, frame),
        Screen::ObjectDetails => {
            view_state.selected_line_details_field_name = render_line_details_screen(model, &mut view_state.line_details_list_state, frame)
        }
        Screen::ValueDetails => render_value_details_screen(model, &mut view_state.value_screen_list_state, frame),
    }

    model.view_state = view_state;
}

fn render_main_screen(
    model: &Model,
    list_state: &mut ListState,
    frame: &mut Frame,
) {
    let block = if model.has_find_task() {
        Block::bordered()
            .title_bottom(Line::from(model.render_find_task_line_left()).bold().left_aligned())
            .title_bottom(Line::from(model.render_find_task_line_right()).bold().right_aligned())
    } else {
        Block::bordered()
            .title_bottom(Line::from(model.render_status_line_left()).left_aligned())
            .title_bottom(Line::from(model.render_status_line_right()).right_aligned())
    };

    let json_line_list = List::new(model)
        .block(block)
        .highlight_style(Style::new().underlined())
        .highlight_symbol("> ")
        .scroll_padding(1);
    frame.render_stateful_widget(json_line_list, frame.area(), list_state);
}

/// returns the key of the selected attribute
fn render_line_details_screen(
    model: &Model,
    list_state: &mut ListState,
    frame: &mut Frame,
) -> Option<String> {
    let line_idx = model
        .view_state
        .main_window_list_state
        .selected()
        .expect("we should find a a selected field");
    let (list_items, keys_in_rendered_order) = model.raw_json_lines.lines[line_idx].render_fields_as_list(&model.props.fields_order);
    let json_field_list = List::new(list_items)
        .block(
            Block::bordered()
                .title_bottom(Line::from(model.render_status_line_left()).left_aligned())
                .title_bottom(Line::from(model.render_status_line_right()).right_aligned()),
        )
        .highlight_style(Style::new().underlined())
        .scroll_padding(1);
    frame.render_stateful_widget(json_field_list, frame.area(), list_state);

    list_state.selected().map(|i| keys_in_rendered_order.get(i).unwrap().to_string())
}

fn render_value_details_screen(
    model: &Model,
    list_state: &mut ListState,
    frame: &mut Frame,
) {
    let line_idx = model
        .view_state
        .main_window_list_state
        .selected()
        .expect("we should find a a selected field");
    let raw_line = &model.raw_json_lines.lines[line_idx].content;

    let lines = if let Value::Object(o) = serde_json::Value::from_str(raw_line).expect("invalid json") {
        let value = o
            .get(
                model
                    .view_state
                    .selected_line_details_field_name
                    .as_ref()
                    .expect("should have a selected field"),
            )
            .expect("key should exist");
        match value {
            Value::String(s) => s.lines().map(|e| Text::raw(format!("{e}"))).collect::<Vec<_>>(),
            _ => vec![Text::raw(format!("{value}"))]
        }
    } else {
        panic!("should find a json object")
    };

    let details_widget = List::new(lines)
        .block(
            Block::bordered()
                .title_bottom(Line::from(model.render_status_line_left()).left_aligned())
                .title_bottom(Line::from(model.render_status_line_right()).right_aligned()),
        ).highlight_style(Style::new().underlined())
        .scroll_padding(1);

    frame.render_stateful_widget(details_widget, frame.area(), list_state);
}
