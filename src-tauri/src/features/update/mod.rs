pub mod command;
mod model;
mod service;

pub fn enabled() -> bool {
    option_env!("APP_ENABLE_UPDATE")
        .map(|value| value != "false")
        .unwrap_or(true)
}
