#![feature(iter_advance_by)]
mod event;
mod model;
mod props;
mod raw_json_lines;
mod tui;

use crate::model::{Model, Screen};
use crate::props::Props;
use crate::raw_json_lines::{RawJsonLines, SourceName};
use anyhow::anyhow;
use clap::Parser;
use std::fs::File;
use std::io;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(version, about, long_about = Some("JSON Lines Viewer - Terminal-UI to view application logs in 'Json line format' or Zip files containing such files.\n\
                                                \n\
                                                Navigation: Cursor keys, PageUp/Down, Enter/Esc.\n\
                                                Search content: Ctrl-f or '/' and navigate to next/previous finding via Cursor Down/Up.\n\
                                                Save current settings: Ctrl-s (e.g. field order. Settings come from commandline arguments and a previously saved config file)"))]
struct Args {
    /// JSON line input files - `.json` or `.zip` files(s) containing `.json` files
    files: Vec<PathBuf>,

    /// fields displayed in-front; separated by comma
    #[arg(short, long, value_delimiter = ',')]
    field_order: Option<Vec<String>>,

    /// suppressed fields; separated by comma
    #[arg(short, long)]
    suppressed_fields: Option<Vec<String>>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let props: Props = init_props(&args)?;

    let lines = load_files(&args.files)?;

    tui::install_panic_hook();
    let mut terminal = tui::init_terminal()?;

    let mut model = Model::new(props, terminal.size().map_err(|e| anyhow!("{e}"))?, &lines);

    while model.active_screen != Screen::Done {
        // Render the current view
        terminal.draw(|f| tui::view(&mut model, f)).map_err(|e| anyhow!("{e}"))?;

        // Handle events and map to a Message
        let mut current_msg = event::handle_event(&model)?;

        // Process updates as long as they return a non-None message
        while let Some(msg) = current_msg {
            let (next_model, next_message) = model.updated(msg);
            model = next_model;
            current_msg = next_message;
        }
    }

    tui::restore_terminal()?;
    Ok(())
}

fn init_props(args: &Args) -> anyhow::Result<Props> {
    let mut props = Props::init()?;
    if let Some(e) = &args.field_order {
        props.fields_order = e.clone();
    }
    if let Some(e) = &args.suppressed_fields {
        props.fields_suppressed = e.clone();
    }
    Ok(props)
}

fn load_files(files: &[PathBuf]) -> anyhow::Result<RawJsonLines> {
    let mut raw_lines = RawJsonLines::default();
    for f in files {
        let path = PathBuf::from(f);
        match path.extension().map(|e| e.to_str()) {
            Some(Some("json")) => load_lines_from_json(&mut raw_lines, &path)?,
            Some(Some("zip")) => load_lines_from_zip(&mut raw_lines, &path)?,
            _ => writeln!(&mut io::stderr(), "unknown file extension: '{}'", path.to_string_lossy()).expect("failed to write to stderr"),
        }
    }

    Ok(raw_lines)
}

fn load_lines_from_json(
    raw_lines: &mut RawJsonLines,
    path: &Path,
) -> anyhow::Result<()> {
    for (line_nr, line) in io::BufReader::new(File::open(path)?).lines().enumerate() {
        raw_lines.push(SourceName::JsonFile(path.file_name().unwrap().to_string_lossy().into()), line_nr + 1, line?);
    }
    Ok(())
}

fn load_lines_from_zip(
    raw_lines: &mut RawJsonLines,
    path: &Path,
) -> anyhow::Result<()> {
    let zip_file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(zip_file)?;

    for i in 0..archive.len() {
        let f = archive.by_index(i)?;
        if f.is_file() && f.name().ends_with(".json") {
            let json_file = f.name().to_string();
            for (line_nr, line) in io::BufReader::new(f).lines().enumerate() {
                raw_lines.push(
                    SourceName::JsonInZip {
                        zip_file: path.file_name().unwrap().to_string_lossy().into(),
                        json_file: json_file.clone(),
                    },
                    line_nr + 1,
                    line?,
                );
            }
        }
    }
    Ok(())
}


// Version 1
// TODO feature: search including jump to NEXT and PREVIOUS hit
//  - search main screen by '/'
//  - search object details screen by '/'
//  - search Value Details screen by '/'

// Version 2
// TODO generalize viewer to any kind of json and any object depth

// Maybe
// TODO maybe feature: settings screen
// TODO maybe feature: highlight lines by text / regexp search string
// TODO maybe feature: Use Memory Mapped Files for RawJsonLines
// TODO maybe feature: possibility to sort lines by one or more field values
// TODO maybe feature: highlight certain field-values
