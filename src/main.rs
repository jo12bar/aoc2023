pub mod app;
pub mod cli;
pub mod fps_counter;
pub mod message;
pub mod model;
pub mod termination;
pub mod tui;
pub mod utils;
pub mod view;

use clap::Parser;
use color_eyre::eyre::Result;

use crate::{
    app::App,
    cli::Cli,
    termination::create_termination,
    utils::{initialize_logging, initialize_panic_handler, version},
};

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(e) = tokio_main().await {
        eprintln!(
            "{} {} error: Something went wrong",
            env!("CARGO_PKG_NAME"),
            version()
        );
        Err(e)
    } else {
        Ok(())
    }
}

async fn tokio_main() -> Result<()> {
    initialize_panic_handler()?;
    initialize_logging()?;

    let (terminator, interrupt_rx) = create_termination();

    let args = Cli::parse();
    let mut app = App::new(
        args.tick_rate,
        args.frame_rate,
        terminator,
        interrupt_rx.resubscribe(),
    )?;
    app.run().await?;

    Ok(())
}
