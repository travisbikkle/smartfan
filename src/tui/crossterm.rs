use std::{error::Error, io};
use std::time::Instant;

use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};

use crate::{tui::app::App, tui::ui, Message};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::mpsc::error::TryRecvError;
use chrono::Local;

pub fn run(
    enhanced_graphics: bool,
    event_receiver_from_ipmi: Receiver<crate::Message>,
    ui_event_sender: Sender<crate::UIMessage>,
) -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new(
        "Crossterm Demo",
        enhanced_graphics,
        event_receiver_from_ipmi,
        ui_event_sender,
    );
    let app_result = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = app_result {
        log::error!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App<'_>) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let tick_rate = std::time::Duration::from_millis(2500);
    loop {

        terminal.draw(|frame| ui::draw(frame, &mut app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Left | KeyCode::Char('h') => app.on_left(),
                        KeyCode::Up | KeyCode::Char('k') => app.on_up(),
                        KeyCode::Right | KeyCode::Char('l') => app.on_right(),
                        KeyCode::Down | KeyCode::Char('j') => app.on_down(),
                        KeyCode::Char(c) => app.on_key(c),
                        _ => {}
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            match app.event_receiver_from_ipmi.try_recv() {
                Ok(msg) => {
                    match msg {
                        Message::Log(l, m) => {
                            app.logs.items.insert(0, (l, m));
                        },
                        Message::Ipmi(temp, speed) => {
                            let now = Local::now();
                            // 将时间格式化为小时:分钟:秒
                            let time_str = now.format("%H:%M:%S").to_string();
                            app.barchart_temp.insert(0, (time_str.clone(), temp as u64));
                            app.barchart_speed.insert(0, (time_str, speed as u64));
                        },
                        _ => {}
                    }
                }
                Err(e) => match e {
                    TryRecvError::Empty => {},
                    TryRecvError::Disconnected => app.logs.items.insert(0, (log::Level::Error, "Shutdown".into())),
                }
            }
            //app.on_tick();
            last_tick = Instant::now();
        }
        if app.should_quit {
            return Ok(());
        }
    }
}
