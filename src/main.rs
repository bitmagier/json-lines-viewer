mod model;
mod event;
mod props;
mod raw_json_lines;

use crate::model::{Model, Screen};
use crate::props::Props;
use clap::Parser;
use std::fs::File;
use std::io;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use ratatui::Frame;
use ratatui::prelude::{Line, Style, Stylize};
use ratatui::widgets::{Block, List, ListState};
use crate::raw_json_lines::{RawJsonLines, SourceName};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    files: Vec<PathBuf>,

    /// fields ordered first - separated by comma
    #[arg(short, long, value_delimiter = ',')]
    field_order: Option<Vec<String>>,

    /// suppressed fields - separated by comma
    #[arg(short, long)]
    suppressed_fields: Option<Vec<String>>
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
        terminal.draw(|f| view(&mut model, f))?;

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


fn load_files(files: &[PathBuf]) -> anyhow::Result<RawJsonLines>
{
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


fn load_lines_from_json(raw_lines: &mut RawJsonLines, path: &Path) -> anyhow::Result<()> {
    for (line_nr, line) in io::BufReader::new(File::open(path)?).lines().enumerate() {
        raw_lines.push(
            SourceName::JsonFile(path.to_path_buf()),
            line_nr + 1,
            line?
        );
    }
    Ok(())
}

fn load_lines_from_zip(raw_lines: &mut RawJsonLines, path: &Path) -> anyhow::Result<()> {
    let zip_file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(zip_file)?;

    for i in 0..archive.len() {
        let f = archive.by_index(i)?;
        if f.is_file() && f.name().ends_with(".json") {
            let json_file = f.name().to_string();
            for (line_nr, line) in io::BufReader::new(f).lines().enumerate() {
                raw_lines.push(
                    SourceName::JsonInZip { zip_file: path.to_path_buf(), json_file: json_file.clone() },
                    line_nr + 1,
                    line?
                );
            }
        }
    }
    Ok(())
}


pub fn view(model: &mut Model, frame: &mut Frame) {
    let mut main_window_list_state = model.main_window_list_state.clone();

    match model.active_screen {
        Screen::Done => (),
        Screen::Main => {
            render_main_screen(model, frame, &mut main_window_list_state);
        }
        Screen::LineDetails => todo!(), // frame.render_widget(DetailScreenWidget::new(), frame.area()),
    }

    model.update_state(main_window_list_state);
}

fn render_main_screen(model: &Model, frame: &mut Frame, list_state: &mut ListState) {
    let list = List::new(model)
        .block(Block::bordered()
            .title_bottom(Line::from(model.render_main_screen_status_line_left()).left_aligned())
            .title_bottom(Line::from(model.render_main_screen_status_line_right()).right_aligned())
        )
        .highlight_style(Style::new().underlined())
        .highlight_symbol("> ")
        .scroll_padding(1);
    frame.render_stateful_widget(list, frame.area(), list_state)
}


mod tui {
    use ratatui::{
        backend::{Backend, CrosstermBackend},
        crossterm::{
            terminal::{
                disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
            },
            ExecutableCommand,
        },
        Terminal,
    };
    use std::{io::stdout, panic};

    pub fn init_terminal() -> anyhow::Result<Terminal<impl Backend>> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        Ok(terminal)
    }

    pub fn restore_terminal() -> anyhow::Result<()> {
        stdout().execute(LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }

    pub fn install_panic_hook() {
        let original_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            stdout().execute(LeaveAlternateScreen).unwrap();
            disable_raw_mode().unwrap();
            original_hook(panic_info);
        }));
    }
}

// TODO implement line detail screen
// TODO feature: filter displayed lines by text / regexp search string
// TODO feature: highlight lines by text / regexp search string
// TODO feature: possibility to sort lines by one or more field values
// TODO feature: Use Memory Mapped Files for RawJsonLines
// TODO feature: render only the visible lines
// TODO maybe feature: highlight certain field-values
