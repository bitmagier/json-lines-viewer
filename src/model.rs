use crate::props::Props;
use crate::raw_json_lines::RawJsonLines;
use ratatui::prelude::{Color, Line, Size, Span, Style, Stylize, Text};
use ratatui::widgets::{Block, List, ListState};
use ratatui::Frame;
use serde_json::{Map, Value};
use std::cell::RefCell;
use std::cmp;

#[derive(Clone)]
pub struct Model<'a> {
    pub active_screen: Screen,
    pub raw_json_lines: &'a RawJsonLines,
    pub props: Props,
    pub main_window_list_state: ListState,
    pub terminal_size: Size,
    max_num_printable_fields: RefCell<usize>,
    line_view_field_offset: usize,
}

#[derive(Clone, Default, Eq, PartialEq)]
pub enum Screen {
    Done,
    #[default]
    Main,
    LineDetails,
}
pub enum Message {
    First,
    Last,
    ScrollUp,
    ScrollDown,
    PageUp,
    PageDown,
    ScrollLeft,
    ScrollRight,
    Enter,
    Exit,
    SaveSettings,
    Resized(Size),
}

impl<'a> Model<'a> {
    pub fn new(props: Props, terminal_size: Size, raw_json_lines: &'a RawJsonLines) -> Self {
        Self {
            active_screen: Default::default(),
            raw_json_lines,
            props,
            main_window_list_state: if raw_json_lines.is_empty() {
                ListState::default()
            } else {
                ListState::default().with_selected(Some(0))
            },
            terminal_size,
            max_num_printable_fields: RefCell::new(0), // gets updated before the first usage
            line_view_field_offset: 0,
        }
    }

    pub fn update_view_state(&mut self, main_window_list_state: ListState) {
        self.main_window_list_state = main_window_list_state
    }

    pub fn updated(mut self, msg: Message) -> (Model<'a>, Option<Message>) {
        match self.active_screen {
            Screen::Done => (self, None),
            Screen::Main => match msg {
                Message::First => {
                    self.main_window_list_state.select_first();
                    (self, None)
                }
                Message::Last => {
                    self.main_window_list_state.select_last();
                    (self, None)
                }
                Message::ScrollUp => {
                    self.main_window_list_state.select_previous();
                    (self, None)
                }
                Message::ScrollDown => {
                    self.main_window_list_state.select_next();
                    (self, None)
                }
                Message::PageUp => {
                    self.main_window_list_state
                        .scroll_up_by(self.terminal_size.height - 2);
                    (self, None)
                }
                Message::PageDown => {
                    self.main_window_list_state
                        .scroll_down_by(self.terminal_size.height - 2);
                    (self, None)
                }
                Message::ScrollLeft => {
                    if self.line_view_field_offset > 0 {
                        self.line_view_field_offset -= 1;
                    }
                    (self, None)
                }
                Message::ScrollRight => {
                    if self.line_view_field_offset + 1 < *self.max_num_printable_fields.borrow() {
                        self.line_view_field_offset += 1;
                    }
                    (self, None)
                }
                Message::Enter => {
                    self.active_screen = Screen::LineDetails;
                    (self, None)
                }
                Message::Exit => {
                    self.active_screen = Screen::Done;
                    (self, None)
                }
                Message::SaveSettings => {
                    self.props.save().expect("failed to save settings");
                    (self, None)
                }
                Message::Resized(size) => {
                    self.terminal_size = size;
                    (self, None)
                }
            },
            Screen::LineDetails => match msg {
                Message::Exit => {
                    self.active_screen = Screen::Main;
                    (self, None)
                }
                _ => (self, None),
            },
        }
    }

    // returns lines and max number of displayed fields
    pub fn render_json_lines(&self) -> Vec<Text> {
        let mut lines = vec![];
        let mut max_num_displayed_fields = 0_usize;
        for line in &self.raw_json_lines.lines {
            let v: Value = serde_json::from_str(&line.content).expect("invalid json");
            match v {
                Value::Object(o) => {
                    let (line, num_fields) = self.render_json_line(o);
                    lines.push(line);
                    max_num_displayed_fields = cmp::max(max_num_displayed_fields, num_fields)
                }
                _ => lines.push(Line::from(format!("{}", v))),
            }
        }
        *self.max_num_printable_fields.borrow_mut() = max_num_displayed_fields;
        lines.into_iter().map(Text::from).collect()
    }

    // returns the line and the number of displayed fields
    fn render_json_line(&self, m: Map<String, Value>) -> (Line, usize) {
        fn render_property(line: &mut Line, k: &str, v: &Value) {
            if line.iter().len() > 0 {
                line.push_span(Span::styled(", ", Color::Gray));
            }
            line.push_span(Span::styled(k.to_string(), Color::Green));
            line.push_span(":".dark_gray());
            line.push_span(format!("{}", v).gray());
        }

        let mut line = Line::default();
        let mut num_fields = 0;
        for k in &self.props.fields_order {
            if let Some(v) = m.get(k) {
                if self.line_view_field_offset <= num_fields {
                    render_property(&mut line, k, v);
                }
                num_fields += 1;
            }
        }

        for (k, v) in &m {
            if !self.props.fields_order.contains(k) && !self.props.fields_suppressed.contains(k) {
                if self.line_view_field_offset <= num_fields {
                    render_property(&mut line, k, v);
                }
                num_fields += 1;
            }
        }
        (line, num_fields)
    }
}

pub fn view(model: &mut Model, frame: &mut Frame) {
    let mut main_window_list_state = model.main_window_list_state.clone();

    match model.active_screen {
        Screen::Done => (),
        Screen::Main => {
            let lines = model.render_json_lines();
            let list = List::new(lines)
                .block(Block::bordered())
                .highlight_style(Style::new().underlined())
                .highlight_symbol("> ")
                .scroll_padding(1);
            frame.render_stateful_widget(list, frame.area(), &mut main_window_list_state)
        }
        Screen::LineDetails => todo!(), // frame.render_widget(DetailScreenWidget::new(), frame.area()),
    }

    model.update_view_state(main_window_list_state);
}
