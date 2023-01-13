pub mod file_editor;
pub mod stateful_list;

use clipboard::{osx_clipboard::OSXClipboardContext, ClipboardContext, ClipboardProvider};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, fs, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Corner, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame, Terminal,
};

struct AppState {
    bookmarks: stateful_list::StatefulList<String>,
    clipboard: OSXClipboardContext,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            bookmarks: stateful_list::StatefulList::with_items(vec![]),
            clipboard: ClipboardContext::new().expect("Unable to connect to clipboard."),
        }
    }
}

const BOOKMARKS_PATH: &str = ".bookmarks";

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let contents = fs::read_to_string(BOOKMARKS_PATH).expect("Unable to open file, ensure there is a .bookmarks file within this projects root.");

    let mut app_state = AppState::default();

    for bookmark in contents.trim().lines() {
        app_state.bookmarks.push(bookmark.to_string());
    }

    let res = run_app(&mut terminal, app_state);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app_state: AppState,
) -> Result<(), Box<dyn Error>> {
    // Select the first item in the list
    app_state.bookmarks.next();
    let mut previous_key: KeyCode = KeyCode::Null;

    loop {
        terminal.draw(|f| ui(f, &mut app_state))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                // Exit keys
                KeyCode::Char('q') => return Ok(()),

                // Bookmark interaction keys
                KeyCode::Char('d') => {
                    if previous_key == KeyCode::Char('d') {
                        if let Some(selected_index) = app_state.bookmarks.selected() {
                            app_state
                                .clipboard
                                .set_contents(app_state.bookmarks.items[selected_index].to_string())
                                .expect("Unable to set clipboard contents.");

                            app_state.bookmarks.delete(selected_index);

                            file_editor::delete_line_from_file(selected_index)?;
                        }
                    }
                }
                KeyCode::Char('p') => {
                    let mut new_item = app_state
                        .clipboard
                        .get_contents()
                        .expect("Unable to get clipboard contents.");

                    if !new_item.ends_with('\n') {
                        new_item.push('\n');
                    }

                    // Todo: Don't clone this
                    file_editor::append_to_file(new_item.clone())?;

                    if app_state.bookmarks.items.is_empty() {
                        app_state.bookmarks.push(new_item)
                    } else {
                        if let Some(selected_index) = app_state.bookmarks.selected() {
                            app_state.bookmarks.insert(new_item, selected_index + 1);
                        }
                    }
                }
                KeyCode::Char('y') => {
                    if previous_key == KeyCode::Char('y') {
                        if let Some(selected_index) = app_state.bookmarks.selected() {
                            app_state
                                .clipboard
                                .set_contents(app_state.bookmarks.items[selected_index].to_string())
                                .expect("Unable to set clipboard contents.");
                        }
                    }
                }

                // Vertical movement keys
                KeyCode::Char('k') | KeyCode::Up => app_state.bookmarks.previous(),
                KeyCode::Char('j') | KeyCode::Down => app_state.bookmarks.next(),
                _ => {}
            }

            if previous_key == KeyCode::Char('d') || previous_key == KeyCode::Char('y') {
                previous_key = KeyCode::Null
            } else {
                previous_key = key.code;
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app_state: &mut AppState) {
    let size = f.size();

    let create_block = |title: &str| {
        Block::default()
            .borders(Borders::ALL)
            .title(title.to_string())
    };

    let chunks = Layout::default()
        .margin(1)
        .constraints([Constraint::Percentage(100)])
        .split(size);

    let bookmarks: Vec<ListItem> = app_state
        .bookmarks
        .items
        .iter()
        .map(|bookmark| ListItem::new(bookmark.as_ref()))
        .collect();

    let bookmarks_list = List::new(bookmarks)
        .block(create_block("Bookmarks - Press ? for help"))
        .highlight_style(Style::default().bg(Color::LightGreen))
        .start_corner(Corner::TopLeft);

    f.render_stateful_widget(bookmarks_list, chunks[0], &mut app_state.bookmarks.state)
}
