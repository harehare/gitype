use anyhow::{anyhow, Result};
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ignore::Walk;
use rand::prelude::*;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

mod app;
mod reader;
mod types;
mod views;
use crate::views::{view, Theme};
use app::App;
use reader::file::FileReader;
use reader::Reader;
use types::typing::Typing;

const QUIT_COMMAND: char = 'q';
const EXIT_COMMAND: char = 'c';
const RESTART_COMMAND: char = 'r';
const ONE_SEC: Duration = Duration::from_secs(1);

#[derive(Parser, Debug)]
#[clap(author, about, long_about = None, version = "v0.1.0")]
struct Args {
    #[clap(long, default_value_t = 30)]
    time: usize,

    #[clap(long, default_value_t = 20)]
    line: usize,

    #[clap(short = 'f', parse(from_os_str), value_name = "file", value_hint = clap::ValueHint::FilePath)]
    file: Option<PathBuf>,

    #[clap(short = 'd', parse(from_os_str), value_name = "dir", value_hint = clap::ValueHint::DirPath)]
    dir: Option<PathBuf>,

    #[clap(short = 'e', long)]
    extension: Option<String>,

    #[clap(short = 't', default_value = "dark")]
    theme: String,
}

fn close_app() -> Result<()> {
    disable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

fn run_app(mut app: App, text: &str, theme: Theme, file: PathBuf) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| view(f, &app, &theme, file.clone()))?;

        let timeout = ONE_SEC
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match app.typing {
                    Typing::BeforeStart(_) => match key.code {
                        KeyCode::Right => {
                            app = app.next_time();
                        }
                        KeyCode::Left => {
                            app = app.prev_time();
                        }
                        KeyCode::Char(QUIT_COMMAND) => {
                            return Ok(());
                        }
                        KeyCode::Char(EXIT_COMMAND) if key.modifiers == KeyModifiers::CONTROL => {
                            return Ok(());
                        }
                        KeyCode::Char(c) => {
                            app = app.start().input(c);
                        }
                        _ => (),
                    },
                    Typing::Running(_) => match key.code {
                        KeyCode::Enter => {
                            app = app.input('\n');
                        }
                        KeyCode::Char(EXIT_COMMAND) if key.modifiers == KeyModifiers::CONTROL => {
                            app = app.finish();
                        }
                        KeyCode::Char(c) => {
                            app = app.input(c);
                        }
                        _ => (),
                    },
                    Typing::Finish(_) => match key.code {
                        KeyCode::Char(RESTART_COMMAND) => app = app.restart(text),
                        KeyCode::Char(QUIT_COMMAND) => {
                            return Ok(());
                        }
                        KeyCode::Char(EXIT_COMMAND) if key.modifiers == KeyModifiers::CONTROL => {
                            return Ok(());
                        }
                        _ => (),
                    },
                }
            }
        }

        if last_tick.elapsed() >= ONE_SEC {
            if let Typing::Running(_) = app.typing {
                app = app.tick();
                last_tick = Instant::now();
            }
        }
    }
}

fn start_typing(file: PathBuf, time: Duration, display_line: usize, theme: Theme) -> Result<()> {
    let reader = FileReader::new(file.clone());
    match reader.load() {
        Ok(text) => {
            let app = App::new(&text, time, display_line)?;
            let res = run_app(app, &text, theme, file);

            if let Err(err) = res {
                return Err(anyhow!(format!("{:?}", err)));
            }

            close_app()?;
            Ok(())
        }
        Err(_) => Err(anyhow!(format!("Failed to load file."))),
    }
}

fn list_files(path: PathBuf, target_extension: Option<String>) -> Vec<PathBuf> {
    Walk::new(path)
        .filter_map(|e| match e {
            Ok(entry) => {
                let extension = entry
                    .path()
                    .extension()
                    .and_then(|f| f.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                let match_extension = |target: Option<String>, e: String| {
                    target.map(|t| e == t.to_lowercase()).unwrap_or(true)
                };

                if entry.file_type().unwrap().is_file()
                    && !extension.is_empty()
                    && match_extension(target_extension.clone(), extension.to_string().clone())
                {
                    Some(entry)
                } else {
                    None
                }
            }
            _ => None,
        })
        .map(|e| e.into_path())
        .collect()
}

fn pick_file(path: PathBuf, target_extension: Option<String>) -> Option<PathBuf> {
    let files = list_files(path, target_extension);

    if files.is_empty() {
        return None;
    }

    let mut rng = rand::thread_rng();
    let file_index = rng.gen_range(0..files.len());
    let file = &files[file_index];
    Some(file.clone())
}

fn main() -> Result<()> {
    let args = Args::parse();

    match (args.file, args.dir) {
        (Some(file), _) => start_typing(
            file.clone(),
            Duration::from_secs(args.time as u64),
            args.line,
            Theme::new(&args.theme),
        ),
        (_, Some(dir)) => match pick_file(dir, args.extension) {
            Some(file) => start_typing(
                file.clone(),
                Duration::from_secs(args.time as u64),
                args.line,
                Theme::new(&args.theme),
            ),
            None => Err(anyhow!(format!("File not found."))),
        },
        _ => match pick_file(PathBuf::from(r"."), args.extension) {
            Some(file) => start_typing(
                file.clone(),
                Duration::from_secs(args.time as u64),
                args.line,
                Theme::new(&args.theme),
            ),
            None => Err(anyhow!(format!("File not found."))),
        },
    }
}
