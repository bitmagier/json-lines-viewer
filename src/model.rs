use crate::props::Props;
use crate::raw_json_lines::RawJsonLines;
use ratatui::prelude::{Color, Line, Size, Span, Stylize};
use ratatui::widgets::{ListItem, ListState};
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
    // returns true for lines to be displayed
    pub json_line_filter: fn(&Map<String, Value>) -> bool,
    num_fields_high_water_mark: RefCell<usize>,
    line_view_field_offset: usize,
    last_action_result: String,
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
    pub fn new(
        props: Props,
        terminal_size: Size,
        raw_json_lines: &'a RawJsonLines,
    ) -> Self {
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
            json_line_filter: |_| true,
            num_fields_high_water_mark: RefCell::new(0), // gets updated before the first usage
            line_view_field_offset: 0,
            last_action_result: String::new(),
        }
    }

    pub fn update_state(
        &mut self,
        main_window_list_state: ListState,
    ) {
        self.main_window_list_state = main_window_list_state
    }

    pub fn updated(
        mut self,
        msg: Message,
    ) -> (Model<'a>, Option<Message>) {
        self.last_action_result.clear();
        match self.active_screen {
            Screen::Done => (self, None),
            Screen::Main => match msg {
                // we need exact instant calculation of the ListState (and cannot rely on lazy corrections e.g. after `ListState::scroll_up_by`),
                // because the pos is used in other render methods
                Message::First => {
                    self.main_window_list_state.select_first();
                    (self, None)
                }
                Message::Last => {
                    self.main_window_list_state
                        .select(Some(cmp::min(self.raw_json_lines.lines.len() as isize - 1, 0) as usize));
                    (self, None)
                }
                Message::ScrollUp => {
                    if let Some(pos) = self.main_window_list_state.selected() {
                        self.main_window_list_state.select(Some(cmp::max(pos as isize - 1, 0) as usize));
                    }
                    (self, None)
                }
                Message::ScrollDown => {
                    if let Some(pos) = self.main_window_list_state.selected() {
                        self.main_window_list_state.select(Some(
                            cmp::min(pos as isize + 1, self.raw_json_lines.lines.len() as isize - 1) as usize
                        ));
                    }
                    (self, None)
                }
                Message::PageUp => {
                    if let Some(pos) = self.main_window_list_state.selected() {
                        self.main_window_list_state
                            .select(Some(cmp::max(pos as isize - self.terminal_size.height as isize - 2, 0) as usize))
                    }
                    (self, None)
                }
                Message::PageDown => {
                    if let Some(pos) = self.main_window_list_state.selected() {
                        self.main_window_list_state.select(Some(cmp::min(
                            pos as isize + self.terminal_size.height as isize - 2,
                            self.raw_json_lines.lines.len() as isize - 1,
                        ) as usize))
                    }
                    (self, None)
                }
                Message::ScrollLeft => {
                    if self.line_view_field_offset > 0 {
                        self.line_view_field_offset -= 1;
                    }
                    (self, None)
                }
                Message::ScrollRight => {
                    if self.line_view_field_offset + 1 < *self.num_fields_high_water_mark.borrow() {
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
                    self.last_action_result = match self.props.save() {
                        Ok(_) => "Ok: settings saved".to_string(),
                        Err(_) => "Error: failed to save settings".to_string(),
                    };
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

    fn render_json_line(
        &self,
        m: Map<String, Value>,
    ) -> Line {
        fn render_property(
            line: &mut Line,
            k: &str,
            v: &Value,
        ) {
            if line.iter().len() > 0 {
                line.push_span(Span::styled(", ", Color::Gray));
            }
            line.push_span(Span::styled(k.to_string(), Color::Green));
            line.push_span(":".dark_gray());
            line.push_span(format!("{v}").gray());
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

        if num_fields > *self.num_fields_high_water_mark.borrow() {
            self.num_fields_high_water_mark.replace(num_fields);
        }
        line
    }

    pub fn render_main_screen_status_line_left(&self) -> String {
        match self.main_window_list_state.selected() {
            None => String::new(),
            Some(line_nr) => {
                let raw_line = &self.raw_json_lines.lines[line_nr];
                let source_name = self.raw_json_lines.source_name(raw_line.source_id).expect("invalid source id");
                format!("{}:{}", source_name, raw_line.line_nr)
            }
        }
    }
    pub fn render_main_screen_status_line_right(&self) -> String {
        self.last_action_result.clone()
    }
}

pub struct ModelIntoIter<'a> {
    model: &'a Model<'a>,
    index: usize,
}

impl<'a> IntoIterator for &'a Model<'_> {
    type Item = ListItem<'a>;
    type IntoIter = ModelIntoIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ModelIntoIter { model: self, index: 0 }
    }
}

impl<'a> Iterator for ModelIntoIter<'a> {
    type Item = ListItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.model.raw_json_lines.lines.len() {
            None
        } else {
            let raw_line = &self.model.raw_json_lines.lines[self.index];
            match serde_json::from_str(&raw_line.content).expect("invalid json") {
                Value::Object(o) => match (self.model.json_line_filter)(&o) {
                    false => {
                        self.index += 1;
                        self.next()
                    }
                    true => {
                        let line = self.model.render_json_line(o);
                        self.index += 1;
                        Some(ListItem::new(line))
                    }
                },
                v => {
                    self.index += 1;
                    Some(ListItem::new(Line::from(format!("{v}"))))
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.model.raw_json_lines.lines.len() - self.index))
    }
}
