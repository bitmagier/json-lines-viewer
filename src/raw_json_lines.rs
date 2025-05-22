use ratatui::prelude::{Span, Style};
use ratatui::widgets::ListItem;
use rustc_hash::FxHashMap;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

#[derive(Default)]
pub struct RawJsonLines {
    sources: FxHashMap<usize, SourceName>,
    pub lines: Vec<RawJsonLine>,
}

impl RawJsonLines {
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn push(
        &mut self,
        source_name: SourceName,
        line_nr: usize,
        content: String,
    ) {
        let source_id = self.source_id(source_name);
        self.lines.push(RawJsonLine {
            source_id,
            line_nr,
            content,
        })
    }

    pub fn source_name(
        &self,
        source_id: usize,
    ) -> Option<&SourceName> {
        self.sources.get(&source_id)
    }

    fn source_id(
        &mut self,
        source_name: SourceName,
    ) -> usize {
        if let Some((k, _)) = self.sources.iter().find(|&(_, v)| v == &source_name) {
            *k
        } else {
            let id = self.sources.len();
            _ = self.sources.insert(id, source_name);
            id
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum SourceName {
    JsonFile(PathBuf),
    JsonInZip { zip_file: PathBuf, json_file: String },
}
impl Display for SourceName {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            SourceName::JsonFile(e) => write!(f, "{}", e.to_string_lossy()),
            SourceName::JsonInZip { zip_file, json_file } => write!(f, "{}/{}", zip_file.to_string_lossy(), json_file),
        }
    }
}
pub struct RawJsonLine {
    pub source_id: usize,
    pub line_nr: usize,
    pub content: String,
}

impl RawJsonLine {
    /// returns ListItems and keys in rendered order
    pub fn render_fields_as_list(&self, key_order: &[String]) -> (Vec<ListItem>, Vec<String>) {
        if let serde_json::Value::Object(o) = serde_json::from_str(&self.content).expect("not a json value") {

            let mut keys_in_rendered_order: Vec<_> = key_order.iter().filter(|&e| o.contains_key(e)).map(|e| e.clone()).collect();
            keys_in_rendered_order.extend(o.keys().filter(|&e| !key_order.contains(e)).cloned());

            let mut list_items = vec![];

            for k in &keys_in_rendered_order {
                list_items.push(Self::render_attribute(k, o.get(k).unwrap()));
            }

            (list_items, keys_in_rendered_order)
        } else {
            panic!("line should be in json object format")
        }
    }

    fn render_attribute<'a, 'b>(key: &'a str, value: &'a serde_json::Value) -> ListItem<'b> {
        ListItem::new(Span::styled(format!("{key} : {value}"), Style::default()))
    }
}
