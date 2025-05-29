#![feature(iter_advance_by)]
mod event;
mod model;
mod props;
mod raw_json_lines;
mod terminal;

use crate::model::{Model, Screen};
use crate::props::Props;
use crate::raw_json_lines::{RawJsonLines, SourceName};
use anyhow::anyhow;
use clap::Parser;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(version, about, long_about = Some("JSON Lines Viewer - Terminal-UI to view application logs in 'Json line format' or Zip files containing such files.\n\
                                                \n\
                                                Navigation: Cursor keys, PageUp/Down, Enter/Esc.\n\
                                                Search content: Ctrl-f or '/' and navigate to next/previous finding via cursor Down/Up.\n\
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
    let props: Props = init_props(&args).context("failed to init props")?;

    let lines = load_files(&args.files).context("failed to load files")?;

    terminal::install_panic_hook();
    let mut terminal = terminal::init_terminal().context("faild to initialize terminal")?;
    let terminal_size = terminal.size().map_err(|e| anyhow!("{e}")).context("failed to get terminal size")?;

    let mut model = Model::new(props, terminal_size, &lines);

    while model.active_screen != Screen::Done {
        // Render the current view
        terminal.draw(|f| terminal::view(&mut model, f)).map_err(|e| anyhow!("{e}")).context("failed to draw to terminal")?;

        // Handle events and map to a Message
        let mut current_msg = event::handle_event(&model)?;

        // Process updates as long as they return a non-None message
        while let Some(msg) = current_msg {
            let (next_model, next_message) = model.updated(msg);
            model = next_model;
            current_msg = next_message;
        }
    }

    terminal::restore_terminal()?;
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

    for path in files {
        match path.extension().and_then(|e| e.to_str()) {
            Some("json") => load_lines_from_json(&mut raw_lines, path).with_context(|| format!("failed to load lines from {path:?}"))?,
            Some("zip") => load_lines_from_zip(&mut raw_lines, path).with_context(|| format!("failed to load lines from {path:?}"))?,
            _ => eprintln!("unknown file extension: '{}'", path.to_string_lossy()),
        }
    }

    Ok(raw_lines)
}

fn load_lines_from_json(
    raw_lines: &mut RawJsonLines,
    path: &Path,
) -> anyhow::Result<()> {
    let json_file = File::open(path).context("failed to open json")?;
    let json_file = io::BufReader::new(json_file);

    for (line_nr, line) in json_file.lines().enumerate() {
        let line = line.context("failed to read json line")?;
        let file_name = path
            .file_name()
            .context("BUG: json path is missing filename")?
            .to_string_lossy()
            .into();
        let source_name = SourceName::JsonFile(file_name);

        raw_lines.push(source_name, line_nr + 1, line);
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
