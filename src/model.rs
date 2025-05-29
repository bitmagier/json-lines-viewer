use crate::props::Props;
use crate::raw_json_lines::RawJsonLines;
use ratatui::prelude::{Color, Line, Size, Stylize};
use ratatui::style::Styled;
use ratatui::text::ToSpan;
use ratatui::widgets::{ListItem, ListState};
use std::cell::Cell;
use std::cmp;
use std::num::NonZero;
use std::ops::Add;
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
    pub object_detail_list_state: ListState,
    pub selected_object_detail_field_name: Option<String>,
    pub value_screen_vertical_scroll_offset: u16,
}
impl Default for ModelViewState {
    fn default() -> Self {
        ModelViewState {
            main_window_list_state: ListState::default().with_selected(Some(0)),
            object_detail_list_state: ListState::default().with_selected(Some(0)),
            selected_object_detail_field_name: None,
            value_screen_vertical_scroll_offset: 0,
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
        self.search_string.push(c);
        self.found = None;
    }
    pub fn remove_search_char(&mut self) {
        self.search_string.pop();
        self.found = None;
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
                            self.find_next(false);
                            (self, None)
                        }
                        Message::Backspace => {
                            self.find_task.as_mut().unwrap().remove_search_char();
                            self.find_next(false);
                            (self, None)
                        }
                        Message::ScrollUp => {
                            self.find_previous();
                            (self, None)
                        }
                        Message::ScrollDown => {
                            self.find_next(true);
                            (self, None)
                        }
                        Message::Enter => {
                            (self, Some(Message::ScrollDown))
                        }
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
                                    self.switch_screen(Screen::ObjectDetails);
                                    self.view_state.object_detail_list_state.select(Some(0));
                                }
                                (self, None)
                            }
                            Message::Exit => {
                                self.switch_screen(Screen::Done);
                                (self, None)
                            }
                            _ => (self, None),
                        },
                        Screen::ObjectDetails => match msg {
                            Message::First => {
                                self.view_state.object_detail_list_state.select_first();
                                (self, None)
                            }
                            Message::Last => {
                                self.view_state.object_detail_list_state.select_last();
                                (self, None)
                            }
                            Message::ScrollUp => {
                                self.view_state.object_detail_list_state.scroll_up_by(1);
                                (self, None)
                            }
                            Message::ScrollDown => {
                                self.view_state.object_detail_list_state.scroll_down_by(1);
                                (self, None)
                            }
                            Message::PageUp => {
                                self.view_state.object_detail_list_state.scroll_up_by(self.page_len());
                                (self, None)
                            }
                            Message::PageDown => {
                                self.view_state.object_detail_list_state.scroll_down_by(self.page_len());
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
                                self.switch_screen(Screen::ValueDetails);
                                (self, None)
                            }
                            Message::Exit => {
                                self.switch_screen(Screen::Main);
                                (self, None)
                            }
                            _ => (self, None),
                        },
                        Screen::ValueDetails => match msg {
                            Message::ScrollUp => {
                                self.view_state.value_screen_vertical_scroll_offset = self.view_state.value_screen_vertical_scroll_offset.saturating_sub(1);
                                (self, None)
                            }
                            Message::ScrollDown => {
                                self.view_state.value_screen_vertical_scroll_offset += 1; // value is corrected during rendering
                                (self, None)
                            }
                            Message::PageUp => {
                                self.view_state.value_screen_vertical_scroll_offset = self.view_state.value_screen_vertical_scroll_offset.saturating_sub(self.page_len());
                                (self, None)
                            }
                            Message::PageDown => {
                                self.view_state.value_screen_vertical_scroll_offset += self.page_len(); // value is corrected during rendering
                                (self, None)
                            }
                            // Message::OpenFindTask => {
                            //     self.find_task = Some(FindTask::default());
                            //     (self, None)
                            // }
                            Message::Exit => {
                                self.switch_screen(Screen::ObjectDetails);
                                (self, None)
                            }
                            _ => (self, None),
                        },
                    }
                }
            }
        }
    }

    fn switch_screen(
        &mut self,
        new_screen: Screen,
    ) {
        self.active_screen = new_screen;
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
                line.push_span(", ");
            }
            line.push_span(k.to_owned().bold());
            line.push_span(":".to_owned());
            line.push_span(format!("{v}"));
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

    /// returns JSON object lines and keys in rendered order
    pub fn produce_line_details_screen_content(&self) -> (Vec<String>, Vec<String>) {
        let line_idx = self.view_state.main_window_list_state.selected().expect("we should find a a selected line");
        self.raw_json_lines.lines[line_idx].produce_rendered_fields_as_list(&self.props.fields_order)
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
            let color = match task.found {
                None => Color::default(),
                Some(false) => Color::Red,
                Some(true) => Color::Green
            };
            " [".to_span().set_style(color)
                .add("Find ".to_span())
                .add("🔍".to_span())
                .add(": ".bold())
                .add(task.search_string.to_span().bold())
                .add("  ] ".to_span().set_style(color)).to_owned()
        } else {
            Line::raw("").to_owned()
        }
    }

    pub fn render_find_task_line_right(&self) -> Line {
        let Some(task) = &self.find_task else {
            return "".into();
        };

        let Some(found) = task.found else {
            return "".into();
        };

        match found {
            true => "found".into(),
            false => "NOT found".into(),
        }
    }

    pub fn page_len(&self) -> u16 {
        self.terminal_size.height.saturating_sub(2)
    }

    fn save_settings(&mut self) {
        self.last_action_result = match self.props.save() {
            Ok(_) => "Ok: settings saved".to_string(),
            Err(_) => "Error: failed to save settings".to_string(),
        };
    }


    fn find_next(&mut self, skip_current_line: bool) {
        let mut find_task = self.find_task.clone().expect("find task should be set");
        if find_task.found.is_none() {
            find_task.found = Some(false);
        };

        match self.active_screen {
            Screen::Done => (),
            Screen::Main => {
                let mut start_line_num = self.view_state.main_window_list_state.selected().unwrap_or(self.view_state.main_window_list_state.offset());
                if skip_current_line {
                    start_line_num += 1
                }
                for (idx, line) in self.raw_json_lines.lines[start_line_num..].iter().enumerate() {
                    if line.content.contains(&find_task.search_string) {
                        find_task.found = Some(true);
                        self.view_state.main_window_list_state.select(Some(start_line_num + idx));
                        break
                    }
                }
            }
            Screen::ObjectDetails => {
                let mut start_line_num = self.view_state.object_detail_list_state.selected().unwrap_or(self.view_state.object_detail_list_state.offset());
                if skip_current_line {
                    start_line_num += 1
                }
                let (lines, field_names) = self.produce_line_details_screen_content();
                for (idx, line) in lines[start_line_num..].iter().enumerate() {
                    if line.contains(&find_task.search_string) {
                        find_task.found = Some(true);
                        self.view_state.object_detail_list_state.select(Some(start_line_num + idx));
                        let selected_field_name = field_names[start_line_num + idx].clone();
                        self.view_state.selected_object_detail_field_name = Some(selected_field_name);
                        break;
                    }
                }
            }
            Screen::ValueDetails => {}
        };

        self.find_task = Some(find_task);
    }

    fn find_previous(&mut self) {
        let mut find_task = self.find_task.clone().expect("find task should be set");
        if find_task.found.is_none() {
            find_task.found = Some(false);
        };

        match self.active_screen {
            Screen::Done => {}
            Screen::Main => {
                let start_line_num = self.view_state.main_window_list_state.selected().unwrap_or(self.view_state.main_window_list_state.offset());
                for (idx, line) in self.raw_json_lines.lines[..start_line_num].iter().rev().enumerate() {
                    if line.content.contains(&find_task.search_string) {
                        find_task.found = Some(true);
                        self.view_state.main_window_list_state.select(Some(start_line_num - 1 - idx));
                        break;
                    }
                }
            }
            Screen::ObjectDetails => {
                let start_line_num = self.view_state.object_detail_list_state.selected().unwrap_or(self.view_state.object_detail_list_state.offset());
                let (lines, field_names) = self.produce_line_details_screen_content();
                for (idx, line) in lines[..start_line_num].iter().rev().enumerate() {
                    if line.contains(&find_task.search_string) {
                        find_task.found = Some(true);
                        self.view_state.object_detail_list_state.select(Some(start_line_num - 1 - idx));
                        let selected_field_name = field_names[start_line_num - 1 - idx].clone();
                        self.view_state.selected_object_detail_field_name = Some(selected_field_name);
                        break;
                    }
                }
            }
            Screen::ValueDetails => {}
        }
        self.find_task = Some(find_task);
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
