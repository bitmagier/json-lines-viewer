use crate::model::{Model, ModelViewState, Screen};
use ratatui::layout::Position;
use ratatui::prelude::{Line, Rect, Style};
use ratatui::widgets::{Block, List, ListState, Paragraph, Wrap};
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
use std::{cmp, io::stdout, panic};

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
    if frame.area().width < 2 || frame.area().height < 2 {
        return; // don't need to render anything here
    }

    let mut view_state: ModelViewState = model.view_state.clone();

    match model.active_screen {
        Screen::Done => (),
        Screen::Main => render_main_screen(model, &mut view_state.main_window_list_state, frame),
        Screen::ObjectDetails => {
            view_state.selected_object_detail_field_name = render_line_details_screen(model, &mut view_state.object_detail_list_state, frame)
        }
        Screen::ValueDetails => render_value_details_screen(model, &mut view_state.value_screen_vertical_scroll_offset, frame),
    }

    model.view_state = view_state;
}

/// Creates the screen border common to all screens.
/// Returns the Border block and the Cursor position (if there is one)
fn produce_screen_border<'a>(frame_area: Rect, model: &'a Model) -> (Block<'a>, Option<Position>) {
    if model.has_find_task() {
        let find_line = model.render_find_task_line_left();
        let cursor_position = Some(Position::new((1 + find_line.width() - 4) as u16, frame_area.bottom() - 1));
        (Block::bordered()
             .title_bottom(find_line.left_aligned())
             .title_bottom(model.render_find_task_line_right().right_aligned()),
         cursor_position)
    } else {
        (Block::bordered()
             .title_bottom(Line::from(model.render_status_line_left()).left_aligned())
             .title_bottom(Line::from(model.render_status_line_right()).right_aligned()),
         None
        )
    }
}

fn render_main_screen(
    model: &Model,
    list_state: &mut ListState,
    frame: &mut Frame,
) {
    let (block, cursor_position) = produce_screen_border(frame.area(), model);
    let json_line_list = List::new(model)
        .block(block)
        .highlight_style(Style::new().underlined())
        .highlight_symbol("> ")
        .scroll_padding(1);
    if let Some(p) = cursor_position {
        frame.set_cursor_position(p)
    }
    frame.render_stateful_widget(json_line_list, frame.area(), list_state);
}

/// returns the key of the selected attribute
fn render_line_details_screen(
    model: &Model,
    list_state: &mut ListState,
    frame: &mut Frame,
) -> Option<String> {
    let (block, cursor_position) = produce_screen_border(frame.area(), model);
    let (list_items, keys_in_rendered_order) = model.produce_line_details_screen_content();
    let json_field_list = List::new(list_items)
        .block(block)
        .highlight_style(Style::new().underlined())
        .scroll_padding(1);
    if let Some(p) = cursor_position {
        frame.set_cursor_position(p)
    }
    frame.render_stateful_widget(json_field_list, frame.area(), list_state);
    list_state.selected().map(|i| keys_in_rendered_order.get(i).unwrap().to_string())
}

fn render_value_details_screen(
    model: &Model,
    vertical_scroll_offset: &mut u16,
    frame: &mut Frame,
) {
    let line_idx = model.view_state.main_window_list_state.selected().expect("we should find a a selected line");
    let raw_line = &model.raw_json_lines.lines[line_idx].content;
    let text = if let Value::Object(o) = serde_json::Value::from_str(raw_line).expect("invalid json") {
        let value = o.get(model.view_state.selected_object_detail_field_name.as_ref().expect("should have a selected field"))
            .expect("key should exist");
        match value {
            Value::String(s) => s.clone(),
            _ => format!("{value}")
        }
    } else {
        panic!("should find a json object")
    };

    // correct scroll line offset – so that current text lines are always on the screen
    let page_len = frame.area().height.saturating_sub(2);
    let max_reasonable_scroll_offset = (text.lines().count() as u16).saturating_sub(page_len);
    *vertical_scroll_offset = cmp::min(*vertical_scroll_offset, max_reasonable_scroll_offset);

    let (block, cursor_position) = produce_screen_border(frame.area(), model);
    let paragraph = Paragraph::new(text)
        .wrap(Wrap::default())
        .block(block)
        .scroll((*vertical_scroll_offset, 0));
    if let Some(p) = cursor_position {
        frame.set_cursor_position(p)
    }
    frame.render_widget(paragraph, frame.area());
}
