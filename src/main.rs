use color_eyre::eyre;
use ratatui::{backend::CrosstermBackend, Terminal};

pub mod app;
pub mod event;
pub mod tui;
pub mod ui;
pub mod update;

use app::App;
use event::{Event, EventHandler};
use tui::Tui;
use update::update;

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let mut app = App::new();

    // Init terminal user interface
    let backend = CrosstermBackend::new(std::io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(16); // 16ms for 60fps drawing
    let mut tui = Tui::new(terminal, events);
    tui.enter()?;

    // Main loop
    while !app.should_quit {
        // Render the user interface
        tui.draw(&mut app)?;
        // Handle events.
        match tui.events.next()? {
            Event::Tick => {},
            Event::Key(key_event) => update(&mut app, key_event),
            Event::Mouse(_) => {},
            Event::Resize(_, _) => {},
        };
    }

    // Exit the user interface
    tui.exit()?;
    Ok(())
}
