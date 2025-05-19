#![feature(iter_advance_by)]
mod event;
mod model;
mod props;
mod raw_json_lines;
mod tui;

use crate::model::{Model, Screen};
use crate::props::Props;
use crate::raw_json_lines::{RawJsonLines, SourceName};
use clap::Parser;
use ratatui::prelude::{Line, Style, Stylize};
use ratatui::widgets::{Block, List, ListState};
use ratatui::Frame;
use std::fs::File;
use std::io;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    files: Vec<PathBuf>,

    /// fields ordered first - separated by comma
    #[arg(short, long, value_delimiter = ',')]
    field_order: Option<Vec<String>>,

    /// suppressed fields - separated by comma
    #[arg(short, long)]
    suppressed_fields: Option<Vec<String>>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let props: Props = init_props(&args)?;

    let lines = load_files(&args.files)?;

    tui::install_panic_hook();
    let mut terminal = tui::init_terminal()?;

    let mut model = Model::new(props, terminal.size()?, &lines);

    while model.active_screen != Screen::Done {
        // Render the current view
        terminal.draw(|f| tui::view(&mut model, f))?;

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
        raw_lines.push(SourceName::JsonFile(path.to_path_buf()), line_nr + 1, line?);
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
                        zip_file: path.to_path_buf(),
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

fn render_main_screen(
    model: &mut Model,
    frame: &mut Frame,
    list_state: &mut ListState,
) {
    let list = List::new(model as &Model)
        .block(
            Block::bordered()
                .title_bottom(Line::from(model.render_main_screen_status_line_left()).left_aligned())
                .title_bottom(Line::from(model.render_main_screen_status_line_right()).right_aligned()),
        )
        .highlight_style(Style::new().underlined())
        .highlight_symbol("> ")
        .scroll_padding(1);
    frame.render_stateful_widget(list, frame.area(), list_state)
}

// TODO implement line detail screen for long messages (stack traces, etc)
// TODO feature: filter displayed lines by certain field values / regexp (e.g. "level=ERROR")
// TODO implement settings screen
// TODO feature: search including jump to NEXT and PREVIOUS hit
// TODO maybe feature: highlight lines by text / regexp search string
// TODO maybe feature: Use Memory Mapped Files for RawJsonLines
// TODO maybe feature: possibility to sort lines by one or more field values
// TODO maybe feature: highlight certain field-values
