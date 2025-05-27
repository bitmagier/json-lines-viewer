use std::cell::Cell;
use crate::props::Props;
use crate::raw_json_lines::RawJsonLines;
use ratatui::prelude::{Line, Size, Stylize};
use ratatui::widgets::{ListItem, ListState};
use std::cmp;
use std::num::NonZero;
use std::rc::Rc;

#[derive(Clone)]
pub struct Model<'a> {
    pub active_screen: Screen,
    pub raw_json_lines: &'a RawJsonLines,
    pub props: Props,
    pub view_state: ModelViewState,
    pub terminal_size: Size,
    num_fields_high_water_mark: Cell<usize>,
    line_rendering_field_offset: usize,
    last_action_result: String,
    find_task: Option<FindTask>,
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

#[derive(Clone, Default)]
pub struct FindTask {
    pub search_string: String,
    pub found: Option<bool>
}
impl FindTask {
    pub fn add_search_char(&mut self, c: char) {
        self.search_string.push(c)
    }
    pub fn remove_search_char(&mut self) {
        self.search_string.pop();
    }
}

#[derive(Clone, Default, Eq, PartialEq)]
pub enum Screen {
    Done,
    #[default]
    Main,
    ObjectDetails,
    ValueDetails,
}

#[derive(Clone, Copy, Eq, PartialEq)]
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
    OpenFindTask,
    CharacterInput(char),
    Backspace,
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
            view_state: Default::default(),
            terminal_size,
            num_fields_high_water_mark: Cell::new(0), // gets updated before the first usage
            line_rendering_field_offset: 0,
            last_action_result: String::new(),
            find_task: None,
        }
    }

    pub fn has_find_task(&self) -> bool {
        self.find_task.is_some()
    }

    pub fn updated(
        mut self,
        msg: Message,
    ) -> (Model<'a>, Option<Message>) {
        self.last_action_result.clear();

        match msg {
            Message::Resized(size) => {
                self.terminal_size = size;
                (self, None)
            }
            Message::SaveSettings => {
                self.save_settings();
                (self, None)
            }
            _ => {
                if self.has_find_task() {
                    match msg {
                        Message::OpenFindTask => {
                            // workaround to enable searching for slashes too
                            (self, Some(Message::CharacterInput('/')))
                        }
                        Message::CharacterInput(c) => {
                            self.find_task.as_mut().unwrap().add_search_char(c);
                            self.find();
                            (self, None)
                        }
                        Message::Backspace => {
                            self.find_task.as_mut().unwrap().remove_search_char();
                            self.find();
                            (self, None)
                        }
                        Message::ScrollUp => {
                            self.find_previous();
                            (self, None)
                        }
                        Message::ScrollDown => {
                            self.find_next();
                            (self, None)
                        }
                        Message::Enter => (self, Some(Message::ScrollDown)),
                        Message::Exit => {
                            self.find_task = None;
                            (self, None)
                        }
                        _ => (self, None),
                    }
                } else {
                    match self.active_screen {
                        Screen::Done => (self, None),
                        Screen::Main => match msg {
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
                            Message::OpenFindTask => {
                                self.find_task = Some(FindTask::default());
                                (self, None)
                            }
                            Message::Enter => {
                                if self.view_state.main_window_list_state.selected().is_some() {
                                    self.switch_to_screen(Screen::ObjectDetails);
                                    self.view_state.line_details_list_state.select(Some(0));
                                }
                                (self, None)
                            }
                            Message::Exit => {
                                self.switch_to_screen(Screen::Done);
                                (self, None)
                            }
                            _ => (self, None),
                        },
                        Screen::ObjectDetails => match msg {
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
                            Message::ScrollLeft | Message::ScrollRight => (self, None),
                            Message::SaveSettings => {
                                self.save_settings();
                                (self, None)
                            }
                            Message::OpenFindTask => {
                                self.find_task = Some(FindTask::default());
                                (self, None)
                            }
                            Message::Enter => {
                                self.switch_to_screen(Screen::ValueDetails);
                                (self, None)
                            }
                            Message::Exit => {
                                self.switch_to_screen(Screen::Main);
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
                            Message::OpenFindTask => {
                                self.find_task = Some(FindTask::default());
                                (self, None)
                            }
                            Message::Exit => {
                                self.switch_to_screen(Screen::ObjectDetails);
                                (self, None)
                            }
                            _ => (self, None),
                        },
                    }
                }
            }
        }
    }

    fn switch_to_screen(
        &mut self,
        screen: Screen,
    ) {
        self.active_screen = screen;
        self.find_task = None;
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

    pub fn render_find_task_line_left(&self) -> Line {
        if let Some(task) = self.find_task.as_ref() {
            Line::from(vec![" [Find 🔍: ".bold(), task.search_string.to_owned().bold(), "  ] ".into()])
        } else {
            Line::raw("")
        }
    }

    pub fn render_find_task_line_right(&self) -> String {
        if let Some(t) = self.find_task.as_ref() {
            if let Some(state) = t.found {
                return match state {
                    true => "found".to_string(),
                    false => "not found".to_string(),
                }
            }
        }
        "".to_string()
    }

    fn page_len(&self) -> u16 {
        self.terminal_size.height.saturating_sub(2)
    }

    fn save_settings(&mut self) {
        self.last_action_result = match self.props.save() {
            Ok(_) => "Ok: settings saved".to_string(),
            Err(_) => "Error: failed to save settings".to_string(),
        };
    }

    fn find(&mut self) {
        self._find(false)
    }

    fn find_next(&mut self) {
        self._find(true)
    }

    fn _find(&mut self, skip_current_line: bool) {
        let find_task = self.find_task.as_mut().expect("find task should be set");
        match self.active_screen {
            Screen::Done => {}
            Screen::Main => {
                let start_line_num = self.view_state.main_window_list_state.selected().unwrap_or(self.view_state.main_window_list_state.offset())
                    + if skip_current_line { 1 } else { 0 };
                for (idx, line) in self.raw_json_lines.lines[start_line_num..].iter().enumerate() {
                    if line.content.contains(&find_task.search_string) {
                        find_task.found = Some(true);
                        self.view_state.main_window_list_state.select(Some(start_line_num + idx));
                        return
                    }
                }
                find_task.found = Some(false);
            }
            Screen::ObjectDetails => {}
            Screen::ValueDetails => {}
        }
    }

    fn find_previous(&mut self) {
        let find_task = self.find_task.as_mut().expect("find task should be set");
        match self.active_screen {
            Screen::Done => {}
            Screen::Main => {
                let start_line_num = self.view_state.main_window_list_state.selected().unwrap_or(self.view_state.main_window_list_state.offset());
                for (idx, line) in self.raw_json_lines.lines[..start_line_num].iter().rev().enumerate() {
                    if line.content.contains(&find_task.search_string) {
                        find_task.found = Some(true);
                        self.view_state.main_window_list_state.select(Some(start_line_num - 1 - idx));
                        return
                    }
                }
                find_task.found = Some(false);
            }
            Screen::ObjectDetails => {}
            Screen::ValueDetails => {}
        }
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
        if self.index >= self.model.raw_json_lines.lines.len() {
            false
        } else {
            self.index += 1;
            true
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

            self.index += 1;
            Some(ListItem::new(line))
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
