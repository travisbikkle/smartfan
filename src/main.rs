use log::Level;
use std::error::Error;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(Level::Error).unwrap();

    let (tx, rx) = mpsc::channel::<smartfan::Message>(100);
    let (ui_tx, ui_rx) = mpsc::channel::<smartfan::UIMessage>(100);

    tokio::task::spawn_blocking(|| {
        log::info!("initiating loop");
        smartfan::init_loop(tx, ui_rx);
    });

    let gui = tokio::task::spawn(async {
        let _ = smartfan::tui::run_tui(rx, ui_tx).await;
    });

    Ok(gui.await?)
}
