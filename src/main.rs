pub mod action;
pub mod app;
pub mod cli;
pub mod components;
pub mod config;
pub mod mode;
pub mod tui;
pub mod utils;

use clap::Parser;
use color_eyre::eyre::Result;

use crate::{
    app::App,
    cli::Cli,
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

    let args = Cli::parse();
    let mut app = App::new(args.tick_rate, args.frame_rate)?;
    app.run().await?;

    Ok(())
}
