mod app;
mod config;
mod fetch;
mod i18n;
mod opml;
mod store;
mod ui;
mod util;

use std::io;
use std::time::{Duration, Instant};

use log::info;

use clap::{Parser, Subcommand};
use crossterm::ExecutableCommand;
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::app::App;
use crate::app::actions::Action;
use crate::app::events::DbWorker;
use crate::app::input::{
    current_modal, handle_help_key, handle_input_mode, handle_key, handle_mouse,
};
use crate::app::state::InputMode;
use crate::config::Config;
use crate::i18n::Lang;
use crate::ui::theme::Theme;

#[derive(Parser)]
#[command(name = "rivulet", about = "A terminal RSS reader")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Import feeds from an OPML file
    Import {
        /// Path to the OPML file
        file: std::path::PathBuf,
    },
    /// Export feeds to an OPML file
    Export {
        /// Path to write the OPML file (prints to stdout if omitted)
        file: Option<std::path::PathBuf>,
    },
}

fn init_logging() {
    if let Ok(dir) = data_dir() {
        let log_path = dir.join("rivulet.log");
        if let Ok(file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
        {
            let _ = simplelog::WriteLogger::init(
                simplelog::LevelFilter::Info,
                simplelog::Config::default(),
                file,
            );
        }
    }
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Import { file }) => return run_import(&file),
        Some(Commands::Export { file }) => return run_export(file.as_deref()),
        None => {}
    }

    init_logging();
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

fn run_import(path: &std::path::Path) -> io::Result<()> {
    let mut file = std::fs::File::open(path)?;
    let outlines = opml::parse_opml(&mut file).map_err(io::Error::other)?;

    let db_path = data_dir()?.join("rivulet.db");
    let repo =
        store::repo::Repo::open(&db_path).map_err(|e| io::Error::other(format!("DB: {e}")))?;

    let existing_feeds = repo
        .list_feeds()
        .map_err(|e| io::Error::other(format!("DB: {e}")))?;
    let existing_urls: std::collections::HashSet<String> =
        existing_feeds.iter().map(|f| f.url.clone()).collect();

    let existing_groups = repo
        .list_groups()
        .map_err(|e| io::Error::other(format!("DB: {e}")))?;
    let mut group_map: std::collections::HashMap<String, i64> = existing_groups
        .iter()
        .map(|g| (g.name.clone(), g.id))
        .collect();
    let mut max_pos = existing_groups
        .iter()
        .map(|g| g.position)
        .max()
        .unwrap_or(-1);

    let now = util::time::now_timestamp();
    let mut imported = 0;
    let mut skipped = 0;

    for outline in &outlines {
        if existing_urls.contains(&outline.url) {
            skipped += 1;
            continue;
        }

        let group_id = if let Some(group_name) = &outline.group {
            if let Some(&id) = group_map.get(group_name) {
                Some(id)
            } else {
                max_pos += 1;
                let new_group = store::models::NewGroup {
                    name: group_name.clone(),
                    position: max_pos,
                    created_at: now,
                };
                let group = repo
                    .create_group(&new_group)
                    .map_err(|e| io::Error::other(format!("DB: {e}")))?;
                group_map.insert(group_name.clone(), group.id);
                Some(group.id)
            }
        } else {
            None
        };

        let new_feed = store::models::NewFeed {
            title: Some(outline.title.clone()),
            url: outline.url.clone(),
            created_at: now,
        };
        let feed = repo
            .create_feed(&new_feed)
            .map_err(|e| io::Error::other(format!("DB: {e}")))?;

        if let Some(gid) = group_id {
            repo.set_feed_group(feed.id, Some(gid))
                .map_err(|e| io::Error::other(format!("DB: {e}")))?;
        }

        imported += 1;
    }

    eprintln!("Imported {imported} feeds ({skipped} skipped, already exist)");
    Ok(())
}

fn run_export(path: Option<&std::path::Path>) -> io::Result<()> {
    let db_path = data_dir()?.join("rivulet.db");
    let repo =
        store::repo::Repo::open(&db_path).map_err(|e| io::Error::other(format!("DB: {e}")))?;

    let feeds = repo
        .list_feeds()
        .map_err(|e| io::Error::other(format!("DB: {e}")))?;
    let groups = repo
        .list_groups()
        .map_err(|e| io::Error::other(format!("DB: {e}")))?;

    if let Some(path) = path {
        let mut file = std::fs::File::create(path)?;
        opml::export_opml(&mut file, &feeds, &groups)?;
        eprintln!("Exported {} feeds to {}", feeds.len(), path.display());
    } else {
        let mut stdout = io::stdout().lock();
        opml::export_opml(&mut stdout, &feeds, &groups)?;
    }

    Ok(())
}

fn data_dir() -> io::Result<std::path::PathBuf> {
    let base = std::env::var("XDG_DATA_HOME")
        .map(std::path::PathBuf::from)
        .ok()
        .or_else(|| dirs::home_dir().map(|h| h.join(".local").join("share")))
        .ok_or_else(|| {
            io::Error::other(
                "Cannot determine data directory: neither $XDG_DATA_HOME nor $HOME is set",
            )
        })?;
    let dir = base.join("rivulet");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let config = Config::load();
    let lang = Lang::from_code(&config.language);
    let db_path = data_dir()?.join("rivulet.db");
    let db =
        DbWorker::start(&db_path).map_err(|error| io::Error::other(format!("DB: {error:?}")))?;
    let mut app = App::new(db, lang, config.recent_days)
        .map_err(|error| io::Error::other(format!("App: {error}")))?;
    let theme = Theme::default();

    info!(
        "Rivulet started (lang={}, recent_days={})",
        config.language, config.recent_days
    );
    let _ = app.dispatch(Action::LoadGroups);
    let _ = app.dispatch(Action::LoadFeeds);
    let _ = app.dispatch(Action::RefreshUnreadCounts);
    let since = app.since_cutoff();
    let _ = app.dispatch(Action::LoadAllEntries {
        unread_only: app.state.unread_only,
        saved_only: app.state.saved_only,
        since,
    });
    let _ = app.dispatch(Action::RefreshFeeds);

    let refresh_interval = if config.refresh_minutes > 0 {
        Some(Duration::from_secs(config.refresh_minutes * 60))
    } else {
        None
    };
    let mut last_refresh = Instant::now();

    loop {
        app.poll_refresh();

        // Auto-refresh feeds periodically
        if let Some(interval) = refresh_interval {
            if last_refresh.elapsed() >= interval && !app.refreshing() {
                let _ = app.dispatch(Action::RefreshFeeds);
                last_refresh = Instant::now();
            }
        }

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

        app.state.flush_feed_rows();
        let modal_state = current_modal(&app.state, &app.lang);
        terminal.draw(|frame| {
            ui::draw(
                frame,
                &mut app.state,
                &theme,
                modal_state,
                app.recent_days,
                &app.lang,
            )
        })?;

        if event::poll(Duration::from_millis(250))? {
            match event::read()? {
                Event::Key(key) => {
                    if app.state.show_help {
                        handle_help_key(&mut app, key);
                        continue;
                    }

                    if app.state.input_mode != InputMode::None {
                        if handle_input_mode(&mut app, key) {
                            app.state.input_mode = InputMode::None;
                            app.state.modal_selection = 0;
                        }
                        continue;
                    }

                    if handle_key(&mut app, key) {
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
