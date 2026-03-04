mod app;
mod config;
mod fetch;
mod i18n;
mod store;
mod ui;
mod util;

use std::io;
use std::time::Duration;

use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::app::actions::Action;
use crate::app::events::DbWorker;
use crate::app::input::{
    current_modal, handle_input_mode, handle_key, handle_mouse, InputMode,
};
use crate::app::App;
use crate::config::Config;
use crate::i18n::Lang;
use crate::ui::theme::Theme;

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal);

    disable_raw_mode()?;
    terminal.backend_mut().execute(DisableMouseCapture)?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn data_dir() -> io::Result<std::path::PathBuf> {
    let base = std::env::var("XDG_DATA_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join(".local")
                .join("share")
        });
    let dir = base.join("rivulet");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let config = Config::load();
    let lang = Lang::from_code(&config.language);
    let db_path = data_dir()?.join("rivulet.db");
    let db = DbWorker::start(&db_path)
        .map_err(|error| io::Error::other(format!("DB: {:?}", error)))?;
    let mut app = App::new(db, lang)
        .map_err(|error| io::Error::other(format!("App: {}", error)))?;
    let theme = Theme::default();

    let _ = app.dispatch(Action::LoadGroups);
    let _ = app.dispatch(Action::LoadFeeds);
    let _ = app.dispatch(Action::RefreshUnreadCounts);
    let _ = app.dispatch(Action::RefreshFeeds);

    let mut input_mode = InputMode::None;
    let mut input_buffer = String::new();
    let mut show_help = false;
    let mut modal_selection: usize = 0;

    loop {
        app.poll_refresh();

        // Auto-clear status after 3s (5s for errors)
        if let Some(set_at) = app.state.status_set_at {
            let timeout = match app.state.status.as_ref().map(|s| s.kind) {
                Some(crate::app::state::StatusKind::Error) => Duration::from_secs(5),
                _ => Duration::from_secs(3),
            };
            if set_at.elapsed() >= timeout {
                let _ = app.dispatch(Action::ClearStatus);
            }
        }

        let modal_state = current_modal(&input_mode, &input_buffer, show_help, app.state.selected_feed.is_some(), modal_selection, &app.state, &app.lang);
        terminal.draw(|frame| ui::draw(frame, &mut app.state, &theme, modal_state, &app.lang))?;

        if event::poll(Duration::from_millis(250))? {
            match event::read()? {
                Event::Key(key) => {
                    if show_help {
                        show_help = false;
                        continue;
                    }

                    if input_mode != InputMode::None {
                        if handle_input_mode(&mut app, key, &mut input_mode, &mut input_buffer, &mut modal_selection) {
                            input_mode = InputMode::None;
                            modal_selection = 0;
                        }
                        continue;
                    }

                    if handle_key(
                        &mut app,
                        key,
                        &mut input_mode,
                        &mut input_buffer,
                        &mut show_help,
                    ) {
                        break;
                    }
                }
                Event::Mouse(mouse) => {
                    let area = terminal.size().unwrap_or_default();
                    handle_mouse(&mut app, mouse, area.into());
                }
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
    }

    Ok(())
}
