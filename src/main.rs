use log::Level;
use std::error::Error;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(Level::Error).unwrap();

    let (tx, rx) = mpsc::channel::<smartfan::Message>(100);
    let (ui_tx, ui_rx) = mpsc::channel::<smartfan::UIMessage>(100);

    let _ = tokio::task::spawn(async {
        log::info!("initiating loop");
        smartfan::init_loop(tx, ui_rx).await;
    });

    // Ok(ipmi_loop.await?)
    smartfan::tui::run_tui(rx, ui_tx)
}
