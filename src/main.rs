use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    //smartfan::init_loop();
    smartfan::tui::run_tui()
}
