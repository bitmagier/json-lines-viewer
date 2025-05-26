use crate::props::Props;
use crate::raw_json_lines::RawJsonLines;
use ratatui::prelude::{Line, Size, Stylize};
use ratatui::widgets::{ListItem, ListState};
use std::cell::{Cell, RefCell};
use std::cmp;
use std::num::NonZero;
use std::rc::Rc;

#[derive(Clone)]
pub struct Model<'a> {
    pub active_screen: Screen,
    pub raw_json_lines: &'a RawJsonLines,
    pub raw_json_line_visibility_cache: RefCell<Vec<Option<bool>>>,
    pub props: Props,
    pub view_state: ModelViewState,
    pub terminal_size: Size,
    // shall return true for lines to be displayed
    json_line_filter: fn(&serde_json::Value) -> bool,
    num_fields_high_water_mark: Cell<usize>,
    line_rendering_field_offset: usize,
    last_action_result: String,
}

#[derive(Clone)]
pub struct ModelViewState {
    pub main_window_list_state: ListState,
    pub line_details_list_state: ListState,
    pub selected_line_details_field_name: Option<String>,
    pub value_screen_list_state: ListState,
}
impl Default for ModelViewState {
    fn default() -> Self {
        ModelViewState {
            main_window_list_state: ListState::default().with_selected(Some(0)),
            line_details_list_state: ListState::default().with_selected(Some(0)),
            selected_line_details_field_name: None,
            value_screen_list_state: ListState::default(),
        }
    }
}

#[derive(Clone, Default, Eq, PartialEq)]
pub enum Screen {
    Done,
    #[default]
    Main,
    LineDetails,
    ValueDetails,
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
            raw_json_line_visibility_cache: RefCell::new(vec![None; raw_json_lines.lines.len()]),
            props,
            view_state: Default::default(),
            terminal_size,
            json_line_filter: |_| true,
            num_fields_high_water_mark: Cell::new(0), // gets updated before the first usage
            line_rendering_field_offset: 0,
            last_action_result: String::new(),
        }
    }

    pub fn set_json_line_filter(
        &mut self,
        json_line_filter: fn(&serde_json::Value) -> bool,
    ) {
        self.json_line_filter = json_line_filter;
        self.raw_json_line_visibility_cache = RefCell::new(vec![None; self.raw_json_lines.lines.len()]);
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
                    self.view_state.main_window_list_state.select_first();
                    (self, None)
                }
                Message::Last => {
                    self.view_state
                        .main_window_list_state
                        .select(Some(cmp::min(self.raw_json_lines.lines.len() as isize - 1, 0) as usize));
                    (self, None)
                }
                Message::ScrollUp => {
                    if let Some(pos) = self.view_state.main_window_list_state.selected() {
                        self.view_state
                            .main_window_list_state
                            .select(Some(cmp::max(pos as isize - 1, 0) as usize));
                    }
                    (self, None)
                }
                Message::ScrollDown => {
                    if let Some(pos) = self.view_state.main_window_list_state.selected() {
                        self.view_state
                            .main_window_list_state
                            .select(Some(
                                cmp::min(pos as isize + 1, self.raw_json_lines.lines.len() as isize - 1) as usize
                            ));
                    }
                    (self, None)
                }
                Message::PageUp => {
                    if let Some(pos) = self.view_state.main_window_list_state.selected() {
                        self.view_state
                            .main_window_list_state
                            .select(Some(pos.saturating_sub(self.page_len() as usize)))
                    }
                    (self, None)
                }
                Message::PageDown => {
                    if let Some(pos) = self.view_state.main_window_list_state.selected() {
                        self.view_state.main_window_list_state.select(Some(cmp::min(
                            pos + self.page_len() as usize,
                            self.raw_json_lines.lines.len().saturating_sub(1),
                        )))
                    }
                    (self, None)
                }
                Message::ScrollLeft => {
                    if self.line_rendering_field_offset > 0 {
                        self.line_rendering_field_offset -= 1;
                    }
                    (self, None)
                }
                Message::ScrollRight => {
                    if self.line_rendering_field_offset + 1 < self.num_fields_high_water_mark.get() {
                        self.line_rendering_field_offset += 1;
                    }
                    (self, None)
                }
                Message::Enter => {
                    if self.view_state.main_window_list_state.selected().is_some() {
                        self.active_screen = Screen::LineDetails;
                        self.view_state.line_details_list_state.select(Some(0));
                    }
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
                Message::First => {
                    self.view_state.line_details_list_state.select_first();
                    (self, None)
                }
                Message::Last => {
                    self.view_state.line_details_list_state.select_last();
                    (self, None)
                }
                Message::ScrollUp => {
                    self.view_state.line_details_list_state.scroll_up_by(1);
                    (self, None)
                }
                Message::ScrollDown => {
                    self.view_state.line_details_list_state.scroll_down_by(1);
                    (self, None)
                }
                Message::PageUp => {
                    self.view_state.line_details_list_state.scroll_up_by(self.page_len());
                    (self, None)
                }
                Message::PageDown => {
                    self.view_state.line_details_list_state.scroll_down_by(self.page_len());
                    (self, None)
                }
                Message::Enter => {
                    self.active_screen = Screen::ValueDetails;
                    (self, None)
                }
                Message::Exit => {
                    self.active_screen = Screen::Main;
                    (self, None)
                }
                _ => (self, None),
            },
            Screen::ValueDetails => match msg {
                Message::ScrollUp => {
                    self.view_state.value_screen_list_state.scroll_up_by(1);
                    (self, None)
                }
                Message::ScrollDown => {
                    self.view_state.value_screen_list_state.scroll_down_by(1);
                    (self, None)
                }
                Message::PageUp => {
                    self.view_state.value_screen_list_state.scroll_up_by(self.page_len());
                    (self, None)
                }
                Message::PageDown => {
                    self.view_state.value_screen_list_state.scroll_down_by(self.page_len());
                    (self, None)
                }
                // Message::ScrollLeft => {
                //     self.view_state.details_scroll_offset.0 = self.view_state.details_scroll_offset.0.saturating_sub(1);
                //     (self, None)
                // }
                // Message::ScrollRight => {
                //     self.view_state.details_scroll_offset.0 += 1; // TODO limit scrolling here to max text line len
                //     (self, None)
                // }
                Message::Exit => {
                    self.active_screen = Screen::LineDetails;
                    (self, None)
                }
                _ => (self, None),
            },
        }
    }

    fn render_json_line<'x>(
        &self,
        m: &serde_json::Map<String, serde_json::Value>,
    ) -> Line<'x> {
        fn render_property(
            line: &mut Line,
            k: &str,
            v: &serde_json::Value,
        ) {
            if line.iter().len() > 0 {
                line.push_span(", ".gray());
            }
            line.push_span(k.to_string().gray());
            line.push_span(":".gray());
            line.push_span(format!("{v}").white());
        }

        let mut line = Line::default();
        let mut num_fields = 0;
        for k in &self.props.fields_order {
            if let Some(v) = m.get(k) {
                if self.line_rendering_field_offset <= num_fields {
                    render_property(&mut line, k, v);
                }
                num_fields += 1;
            }
        }

        for (k, v) in m {
            if !self.props.fields_order.contains(k) && !self.props.fields_suppressed.contains(k) {
                if self.line_rendering_field_offset <= num_fields {
                    render_property(&mut line, k, v);
                }
                num_fields += 1;
            }
        }

        if num_fields > self.num_fields_high_water_mark.get() {
            self.num_fields_high_water_mark.replace(num_fields);
        }

        line
    }

    pub fn render_status_line_left(&self) -> String {
        match self.view_state.main_window_list_state.selected() {
            Some(line_nr) if self.raw_json_lines.lines.len() > line_nr => {
                let raw_line = &self.raw_json_lines.lines[line_nr];
                let source_name = self.raw_json_lines.source_name(raw_line.source_id).expect("invalid source id");
                format!("{}:{}", source_name, raw_line.line_nr)
            }
            _ => String::new(),
        }
    }
    pub fn render_status_line_right(&self) -> String {
        self.last_action_result.clone()
    }

    pub fn determine_line_visibility<F>(
        &self,
        line_idx: usize,
        json_object_getter: F,
    ) -> bool
    where
        F: FnOnce() -> Rc<serde_json::Value>,
    {
        let val = self.raw_json_line_visibility_cache.borrow()[line_idx];
        match val {
            Some(val) => val,
            None => {
                let result = (self.json_line_filter)(&json_object_getter());
                (*self.raw_json_line_visibility_cache.borrow_mut())[line_idx] = Some(result);
                result
            }
        }
    }

    fn page_len(&self) -> u16 {
        self.terminal_size.height.saturating_sub(2)
    }
}

pub struct ModelIntoIter<'a> {
    model: &'a Model<'a>,
    index: usize,
}

impl<'a> ModelIntoIter<'a> {
    // light version of Self::next() that simply skips the item.
    // returns true if the item was skipped, false if there are no more items
    fn skip_item(&mut self) -> bool {
        fn deserialize(raw: &str) -> serde_json::Value {
            serde_json::from_str(raw).expect("invalid_json")
        }

        if self.index >= self.model.raw_json_lines.lines.len() {
            false
        } else {
            let raw_line = &self.model.raw_json_lines.lines[self.index];
            let is_visible = self
                .model
                .determine_line_visibility(self.index, || Rc::new(deserialize(&raw_line.content)));

            match is_visible {
                false => {
                    self.index += 1;
                    self.skip_item()
                }
                true => {
                    self.index += 1;
                    true
                }
            }
        }
    }
}

impl<'a> IntoIterator for &'a Model<'a> {
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
            let json: Rc<serde_json::Value> = Rc::new(serde_json::from_str(&raw_line.content).expect("invalid json"));
            let line = match json.as_ref() {
                serde_json::Value::Object(o) => self.model.render_json_line(o),
                e => Line::from(format!("{e}")),
            };

            match self.model.determine_line_visibility(self.index, || Rc::clone(&json)) {
                false => {
                    self.index += 1;
                    self.next()
                }
                true => {
                    self.index += 1;
                    Some(ListItem::new(line))
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.model.raw_json_lines.lines.len() - self.index))
    }

    fn advance_by(
        &mut self,
        n: usize,
    ) -> Result<(), NonZero<usize>> {
        for i in 0..n {
            match self.skip_item() {
                true => (),
                false => return Err(NonZero::new(n - i).unwrap()),
            }
        }
        Ok(())
    }
}
