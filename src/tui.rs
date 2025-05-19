use crate::model::{Model, ModelViewState, Screen};
use crate::raw_json_lines::RawJsonLines;
use ratatui::prelude::{Line, Style, Stylize, Text};
use ratatui::widgets::{Block, List, ListState, Paragraph};
use ratatui::{
    backend::{Backend, CrosstermBackend}, crossterm::{
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    Frame,
    Terminal,
};
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
    let ModelViewState {
        mut main_window_list_state,
        // mut line_details_list_state,
    } = model.view_state.clone();

    match model.active_screen {
        Screen::Done => (),
        Screen::Main => render_main_screen(model, &mut main_window_list_state, frame),
        Screen::LineDetails => {
            let line_idx = main_window_list_state.selected().expect("we should find a a selected field");
            render_line_details(model, line_idx, frame);
            // render_line_details_screen(model, line_idx, &mut line_details_list_state, frame)
        }
    }

    model.view_state = ModelViewState {
        main_window_list_state,
        // line_details_list_state,
    };
}

fn render_main_screen(
    model: &Model,
    list_state: &mut ListState,
    frame: &mut Frame,
) {
    let json_line_list = List::new(model)
        .block(
            Block::bordered()
                .title_bottom(Line::from(model.render_status_line_left()).left_aligned())
                .title_bottom(Line::from(model.render_status_line_right()).right_aligned()),
        )
        .highlight_style(Style::new().underlined())
        .highlight_symbol("> ")
        .scroll_padding(1);
    frame.render_stateful_widget(json_line_list, frame.area(), list_state);
}

fn render_line_details(
    model: &Model,
    line_idx: usize,
    frame: &mut Frame,
) {
    frame.render_widget(
        Paragraph::new(render_lines_screen_content(model.raw_json_lines, line_idx)).block(
            Block::bordered()
                .title_bottom(Line::from(model.render_status_line_left()).left_aligned())
                .title_bottom(Line::from(model.render_status_line_right()).right_aligned()),
        ),
        frame.area(),
    );
}

fn render_lines_screen_content(
    raw_json_lines: &RawJsonLines,
    line_idx: usize,
) -> Text {
    let j: serde_json::Value = serde_json::from_str(&raw_json_lines.lines[line_idx].content).expect("should be json");
    Text::raw(format!("{j}"))
}

// fn render_line_details_screen(
//     model: &Model,
//     line_idx: usize,
//     list_state: &mut ListState,
//     frame: &mut Frame,
// ) {
//     let json_field_list = List::new(model.raw_json_lines.lines[line_idx].render_fields_as_list(&model.props.fields_order))
//         .block(
//             Block::bordered()
//                 .title_bottom(Line::from(model.render_main_screen_status_line_left()).left_aligned())
//                 .title_bottom(Line::from(model.render_main_screen_status_line_right()).right_aligned()),
//         )
//         .highlight_style(Style::new().underlined())
//         // .highlight_symbol("")
//         .scroll_padding(1);
//     frame.render_stateful_widget(json_field_list, frame.area(), list_state);
// }
