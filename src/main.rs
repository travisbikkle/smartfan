use log::Level;
use std::error::Error;
use tokio::sync::mpsc;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(Level::Error).unwrap();

    // ipmitool enterprise-numbers -> ${HOME}/.local/usr/share/misc/enterprise-numbers
    #[cfg(target_os = "windows")]
    {
        let exe_path = env::current_exe()?;
        let home_dir = exe_path.parent().unwrap().to_str().unwrap();
        env::set_var("HOME", home_dir);
    }

    let (tx, rx) = mpsc::channel::<smartfan::Message>(100);
    let (ui_tx, ui_rx) = mpsc::channel::<smartfan::UIMessage>(100);

    let _ = tokio::task::spawn(async {
        log::info!("initiating loop");
        smartfan::init_loop(tx, ui_rx).await;
    });

    // Ok(ipmi_loop.await?)
    smartfan::tui::run_tui(rx, ui_tx)
}
