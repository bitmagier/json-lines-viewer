use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use rustc_hash::FxHashMap;

pub struct RawJsonLines {
    sources: FxHashMap<usize, SourceName>,
    pub lines: Vec<RawJsonLine>
}
impl RawJsonLines {
    pub fn new() -> Self {
        Self {
            sources: FxHashMap::default(),
            lines: vec![]
        }
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn push(&mut self, source_name: SourceName, line_nr: usize, content: String) {
        let source_id = self.source_id(source_name);
        self.lines.push(RawJsonLine {
            source_id,
            line_nr,
            content,
        })
    }
    
    pub fn source_name(&self, source_id: usize) -> Option<&SourceName> {
        self.sources.get(&source_id)
    }

    fn source_id(&mut self, source_name: SourceName) -> usize {
        if let Some((k,_)) = self.sources.iter().find(|&(_,v)| v == &source_name) {
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
    JsonInZip {zip_file: PathBuf, json_file: String }
}
impl Display for SourceName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceName::JsonFile(e) => write!(f, "{}", e.to_string_lossy()),
            SourceName::JsonInZip { zip_file, json_file } => write!(f, "{}/{}", zip_file.to_string_lossy(), json_file)
        }
    }
}
pub struct RawJsonLine {
    pub source_id: usize,
    pub line_nr: usize,
    pub content: String
}