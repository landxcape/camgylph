mod app;
mod ascii;
mod camera;
mod config;
mod error;
mod frame;
mod metrics;
mod terminal;

use config::Config;
use error::AppError;

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), AppError> {
    let config = Config::from_env()?;
    if config.show_help {
        config::print_help();
        return Ok(());
    }

    app::run(config)
}
