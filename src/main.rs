mod app;
mod config;
mod pricing;
mod rdap;
mod search;
mod ui;

use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::App;
use pricing::PricingDb;

#[tokio::main]
async fn main() -> Result<()> {
    let pricing_db = init_pricing().await;

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, pricing_db).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = res {
        eprintln!("Error: {e}");
    }

    Ok(())
}

async fn init_pricing() -> Option<Arc<PricingDb>> {
    match pricing::load_cached_pricing() {
        Some(db) => {
            let db = Arc::new(db);
            let weak = Arc::downgrade(&db);
            tokio::spawn(async move {
                if let Ok(_fresh) = pricing::fetch_pricing().await {
                    if let Some(_app) = weak.upgrade() {
                        // live-swap could go here
                    }
                }
            });
            Some(db)
        }
        None => match pricing::fetch_pricing().await {
            Ok(db) => Some(Arc::new(db)),
            Err(e) => {
                eprintln!("warning: could not fetch pricing data: {e}");
                None
            }
        },
    }
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    pricing_db: Option<Arc<PricingDb>>,
) -> Result<()> {
    let mut app = App::new(pricing_db);

    loop {
        if app.loading {
            terminal.draw(|frame| {
                ui::draw(frame, &app);
            })?;
            app.tick_notification();
            app.run_search().await;
            terminal.draw(|frame| {
                ui::draw(frame, &app);
            })?;
            app.tick_notification();
        }

        terminal.draw(|frame| {
            ui::draw(frame, &app);
        })?;
        app.tick_notification();

        if event::poll(Duration::from_millis(50))? {
            let event = event::read()?;
            match event {
                Event::Key(key) if key.kind == event::KeyEventKind::Press => {
                    match (key.code, key.modifiers) {
                        (KeyCode::Char('c'), KeyModifiers::CONTROL)
                        | (KeyCode::Char('q'), KeyModifiers::NONE)
                        | (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                            return Ok(());
                        }
                        _ => {
                            app.handle_key(key)?;
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
