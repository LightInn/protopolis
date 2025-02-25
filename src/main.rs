// main.rs
mod action;
mod agent;
mod app;
mod config;
mod message;
mod personality;
mod prompt;
mod simulation;
mod state;
mod ratatui;

use crate::app::App;
use std::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut terminal = ratatui::init()?;
    let app_result = App::default().run(&mut terminal).await;
    ratatui::restore()?;
    app_result
}