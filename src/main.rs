#![allow(non_snake_case)]
use std::io;

use dirs::data_local_dir;
use tui::backend::CrosstermBackend;
use tui::Terminal;
use CuTE::app::{App, AppResult};
use CuTE::events::event::{Event, EventHandler};
use CuTE::events::handler::handle_key_events;
use CuTE::ui::tui::Tui;

fn main() -> AppResult<()> {
    let mut app = App::new();

    let cutepath = data_local_dir().expect("Failed to get data local directory");
    let cutepath = cutepath.join("CuTE");
    // Check if the directory exists
    if !cutepath.exists() {
        // If it doesn't exist, create it
        if let Err(err) = std::fs::create_dir_all(&cutepath) {
            let dbpath = cutepath.join("CuTE.db");
            std::fs::File::create(&dbpath).expect("failed to create database");
            eprintln!("Failed to create CuTE directory: {}", err);
        } else {
            println!("CuTE directory created at {:?}", cutepath);
        }
    } else {
        println!("CuTE directory already exists at {:?}", cutepath);
    }
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    while app.running {
        tui.draw(&mut app)?;
        match tui.events.next()? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    tui.exit()?;
    Ok(())
}
