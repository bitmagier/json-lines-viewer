use rustc_hash::FxHashMap;
use std::fmt::{Display, Formatter};

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
    JsonFile(String),
    JsonInZip { zip_file: String, json_file: String },
}
impl Display for SourceName {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            SourceName::JsonFile(e) => write!(f, "{e}"),
            SourceName::JsonInZip { zip_file, json_file } => write!(f, "{zip_file}/{json_file}"),
        }
    }
}
pub struct RawJsonLine {
    pub source_id: usize,
    pub line_nr: usize,
    pub content: String,
}

impl RawJsonLine {
    /// returns JSON object lines and keys in rendered order
    pub fn produce_rendered_fields_as_list(&self, key_order: &[String]) -> (Vec<String>, Vec<String>) {
        let value = serde_json::from_str(&self.content).expect("not a json value");

        let serde_json::Value::Object(o) = value else {
            panic!("line should be in json object format")
        };

        let mut keys_in_rendered_order: Vec<_> = key_order.iter().filter(|&e| o.contains_key(e)).cloned().collect();
        keys_in_rendered_order.extend(o.keys().filter(|&e| !key_order.contains(e)).cloned());

        let mut list_items = vec![];

        for k in &keys_in_rendered_order {
            list_items.push(Self::render_attribute(k, o.get(k).unwrap()));
        }

        (list_items, keys_in_rendered_order)
    }

    fn render_attribute(key: &str, value: &serde_json::Value) -> String {
        format!("{key} : {value}")
    }
}
