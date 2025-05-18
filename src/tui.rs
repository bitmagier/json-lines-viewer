use crate::model::{Model, Screen};
use ratatui::{
    backend::{Backend, CrosstermBackend}, crossterm::{
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    Frame,
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

pub fn view(
    model: &mut Model,
    frame: &mut Frame,
) {
    let mut main_window_list_state = model.main_window_list_state.clone();

    match model.active_screen {
        Screen::Done => (),
        Screen::Main => {
            crate::render_main_screen(model, frame, &mut main_window_list_state);
        }
        Screen::LineDetails => todo!(), // frame.render_widget(DetailScreenWidget::new(), frame.area()),
    }

    model.update_state(main_window_list_state);
}
