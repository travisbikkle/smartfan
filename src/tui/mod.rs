use std::error::Error;

use tokio::sync::mpsc::{Receiver, Sender};

use argh::FromArgs;

pub mod app;
pub mod crossterm;
pub mod ui;

/// Demo
#[derive(Debug, FromArgs)]
struct Cli {
    /// whether unicode symbols are used to improve the overall look of the app
    #[argh(option, default = "true")]
    enhanced_graphics: bool,
}

pub fn run_tui(
    event_receiver_from_ipmi: Receiver<crate::Message>,
    ui_event_sender: Sender<crate::UIMessage>,
) -> Result<(), Box<dyn Error>> {
    let cli: Cli = argh::from_env();
    //let tick_rate = Duration::from_millis(cli.tick_rate);
    crossterm::run(
        cli.enhanced_graphics,
        event_receiver_from_ipmi,
        ui_event_sender,
    ).expect("cross term run successfully");
    Ok(())
}
