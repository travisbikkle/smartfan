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
        "灵蛛smartfan",
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
    let tick_rate = std::time::Duration::from_millis(1000);
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
                        Message::Log(time, l, m) => {
                            app.logs.items.insert(0, (l, format!("{} {}", time, m)));
                        },
                        Message::GotCpuAndFansSpeed(time_str, (cpu, max_cpu), fans) => {
                            app.barchart_speed.clear();
                            fans
                                .iter()
                                .for_each(|(fan_name, speed)|app.barchart_temp.push((fan_name.clone(), *speed as u64)))
                        }
                        Message::SetFanSpeed(time_str, temp, speed) => {
                            let max = app.signals.window[1] - app.signals.window[0];
                            if app.signals.data1.len() > max as usize {
                                app.signals.data1.remove(0);
                            }
                            app.signals.data1.push((time_str.clone(), temp));

                            if app.signals.data2.len() > max as usize {
                                app.signals.data2.remove(0);
                            }
                            app.signals.data2.push((time_str.clone(), speed as f64));

                            if app.speed_list.items.len() > 50 {
                                app.speed_list.items.pop();
                            }
                            app.speed_list.items.insert(0, (time_str.clone(), speed));
                            if app.temp_list.items.len() > 50 {
                                app.temp_list.items.pop();
                            }
                            app.temp_list.items.insert(0, (time_str.clone(), temp));
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
