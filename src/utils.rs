use std::path::PathBuf;

use color_eyre::eyre::Result;
use directories::ProjectDirs;
use lazy_static::lazy_static;
use tracing::error;
use tracing_error::ErrorLayer;
use tracing_subscriber::{self, prelude::*, util::SubscriberInitExt, Layer};

/// The git commit (and potentially tag) from build time.
pub static GIT_COMMIT_HASH: &str = env!("_GIT_INFO");

lazy_static! {
    /// The uppercased name of the application from build time.
    pub static ref PROJECT_NAME: String = env!("CARGO_CRATE_NAME").to_uppercase().to_string();
    /// The name of the app's data folder.
    pub static ref DATA_FOLDER: Option<PathBuf> = std::env::var(format!("{}_DATA", PROJECT_NAME.clone())).ok().map(PathBuf::from);
    /// The name of the app's config folder.
    pub static ref CONFIG_FOLDER: Option<PathBuf> = std::env::var(format!("{}_CONFIG", PROJECT_NAME.clone())).ok().map(PathBuf::from);
    /// The name of the environment variable to read for log levels.
    pub static ref LOG_ENV: String = format!("{}_LOG_LEVEL", PROJECT_NAME.clone());
    /// The name of the file to store logs in.
    pub static ref LOG_FILE: String = format!("{}.log", env!("CARGO_PKG_NAME"));
}

/// The directory to store all the project .config and .data dirs in.
///
/// Returns `None` if the home directory could not be resolved at runtime.
fn project_directory() -> Option<ProjectDirs> {
    ProjectDirs::from("ca", "jo12bar", env!("CARGO_PKG_NAME"))
}

/// Initialize the panic handler.
pub fn initialize_panic_handler() -> Result<()> {
    // Setup the color_eyre hooks.
    let (panic_hook, eyre_hook) = color_eyre::config::HookBuilder::default()
        .panic_section(format!(
            "This is a bug. Consider reporting it at {}",
            env!("CARGO_PKG_REPOSITORY")
        ))
        .capture_span_trace_by_default(false)
        .display_location_section(false)
        .display_env_section(false)
        .into_hooks();

    // Setup the eyre report-handler hook.
    eyre_hook.install()?;

    // Wrap the actual eyre hook in our own hook so we can customize it a lot.
    std::panic::set_hook(Box::new(move |panic_info| {
        // Try to exit the terminal UI properly on panic.
        if let Ok(mut t) = crate::tui::Tui::new() {
            if let Err(r) = t.exit() {
                error!("Unable to exit terminal UI: {:?}", r);
            }
        }

        // In release mode, produce a user-friendly panic.
        #[cfg(not(debug_assertions))]
        {
            use human_panic::{handle_dump, print_msg, Metadata};
            let meta = Metadata {
                version: env!("CARGO_PKG_VERSION").into(),
                name: env!("CARGO_PKG_NAME").into(),
                authors: env!("CARGO_PKG_AUTHORS").replace(':', ", ").into(),
                homepage: env!("CARGO_PKG_HOMEPAGE").into(),
            };

            let file_path = handle_dump(&meta, panic_info);
            // prints human-panic message
            print_msg(file_path, &meta)
                .expect("human-panic: printing error message to console failed");
            eprintln!("{}", panic_hook.panic_report(panic_info)); // prints color-eyre stack trace to stderr
        }

        // Print eyre panic report, and log it to our logging infrastructure (without colours).
        let msg = format!("{}", panic_hook.panic_report(panic_info));
        tracing::error!("Error: {}", strip_ansi_escapes::strip_str(msg));

        // In debug mode, print a *really* detailed stack trace.
        #[cfg(debug_assertions)]
        {
            better_panic::Settings::auto()
                .most_recent_first(false)
                .lineno_suffix(true)
                .verbosity(better_panic::Verbosity::Full)
                .create_panic_handler()(panic_info);
        }
    }));

    Ok(())
}

/// Resolve the location of the `.data/` directory.
pub fn get_data_dir() -> PathBuf {
    let directory = if let Some(s) = DATA_FOLDER.clone() {
        s
    } else if let Some(proj_dirs) = project_directory() {
        proj_dirs.data_local_dir().to_path_buf()
    } else {
        PathBuf::from(".").join(".data")
    };
    directory
}

/// Resolve the location of the `.config/` directory.
pub fn get_config_dir() -> PathBuf {
    let directory = if let Some(s) = CONFIG_FOLDER.clone() {
        s
    } else if let Some(proj_dirs) = project_directory() {
        proj_dirs.config_local_dir().to_path_buf()
    } else {
        PathBuf::from(".").join(".config")
    };
    directory
}

/// Initialize the logging infrastructure.
pub fn initialize_logging() -> Result<()> {
    // Make sure the .data directory exists
    let directory = get_data_dir();
    std::fs::create_dir_all(directory.clone())?;

    // Store logs inside the .data directory.
    let log_path = directory.join(LOG_FILE.clone());
    let log_file = std::fs::File::create(log_path)?;

    // Make sure the RUST_LOG environment variable matches whatever is set by
    // AOC2023_LOG_LEVEL (or RUST_LOG if the user set that for some reason)
    std::env::set_var(
        "RUST_LOG",
        std::env::var("RUST_LOG")
            .or_else(|_| std::env::var(LOG_ENV.clone()))
            .unwrap_or_else(|_| format!("{}=info", env!("CARGO_CRATE_NAME"))),
    );

    // Send logs to the log file.
    let file_subscriber = tracing_subscriber::fmt::layer()
        .json()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_writer(log_file)
        .with_target(false)
        .with_ansi(false)
        .with_span_list(true)
        .with_level(true)
        .with_filter(tracing_subscriber::filter::EnvFilter::from_default_env());

    // Also print things to stdout.
    let stdout_subscriber = tracing_subscriber::fmt::layer()
        .pretty()
        .with_ansi(true)
        .with_level(true)
        .with_filter(tracing_subscriber::filter::EnvFilter::from_default_env());

    tracing_subscriber::registry()
        .with(file_subscriber)
        .with(stdout_subscriber)
        .with(ErrorLayer::default())
        .init();

    Ok(())
}

/// Similar to the `std::dbg!` macro, but generates `tracing` events rather
/// than printing to stdout.
///
/// By default, the verbosity level for the generated events is `DEBUG`, but
/// this can be customized.
#[macro_export]
macro_rules! trace_dbg {
    (target: $target:expr, level: $level:expr, $ex:expr) => {{
        match $ex {
            value => {
                tracing::event!(target: $target, $level, ?value, stringify!($ex));
                value
            }
        }
    }};
    (level: $level:expr, $ex:expr) => {
        trace_dbg!(target: module_path!(), level: $level, $ex)
    };
    (target: $target:expr, $ex:expr) => {
        trace_dbg!(target: $target, level: tracing::Level::DEBUG, $ex)
    };
    ($ex:expr) => {
        trace_dbg!(level: tracing::Level::DEBUG, $ex)
    };
}

/// Return a version printout string.
pub fn version() -> String {
    let author = clap::crate_authors!();

    let commit_hash = GIT_COMMIT_HASH;

    let current_exe_path = PathBuf::from(clap::crate_name!()).display().to_string();
    let config_dir_path = get_config_dir().display().to_string();
    let data_dir_path = get_data_dir().display().to_string();

    format!(
        "\
{current_exe_path}
{commit_hash}

Authors: {author}

Config directory: {config_dir_path}
Data directory: {data_dir_path}"
    )
}
