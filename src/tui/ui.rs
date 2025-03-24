use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{self, Span},
    widgets::{
        canvas::{self, Canvas, Circle, Map, MapResolution, Rectangle},
        Axis, BarChart, Block, Cell, Chart, Dataset, List, ListItem, Paragraph, Row, Table, Tabs,
        Wrap,
    },
    Frame,
};

use crate::tui::app::App;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(frame.area());
    let tabs = app
        .tabs
        .titles
        .iter()
        .map(|t| text::Line::from(Span::styled(*t, Style::default().fg(Color::Green))))
        .collect::<Tabs>()
        .block(Block::bordered().title(app.title))
        .highlight_style(Style::default().fg(Color::Yellow))
        .select(app.tabs.index);
    frame.render_widget(tabs, chunks[0]);
    match app.tabs.index {
        0 => draw_first_tab(frame, app, chunks[1]),
        1 => draw_second_tab(frame, app, chunks[1]),
        2 => draw_third_tab(frame, app, chunks[1]),
        _ => {}
    };
}

fn draw_first_tab(frame: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::vertical([
        Constraint::Length(9),
        Constraint::Min(8),
        Constraint::Length(7),
    ])
    .split(area);
    draw_charts(frame, app, chunks[1]);
    draw_text(frame, chunks[2]);
}

#[allow(clippy::too_many_lines)]
fn draw_charts(frame: &mut Frame, app: &mut App, area: Rect) {
    let constraints = if app.show_chart {
        vec![Constraint::Percentage(50), Constraint::Percentage(50)]
    } else {
        vec![Constraint::Percentage(100)]
    };
    let chunks = Layout::horizontal(constraints).split(area);
    {
        let chunks = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[0]);
        {
            let chunks =
                Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(chunks[0]);

            // Draw tasks
            let tasks: Vec<ListItem> = app
                .tasks
                .items
                .iter()
                .map(|i| ListItem::new(vec![text::Line::from(Span::raw(*i))]))
                .collect();
            let tasks = List::new(tasks)
                .block(Block::bordered().title("List"))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                .highlight_symbol("> ");
            frame.render_stateful_widget(tasks, chunks[0], &mut app.tasks.state);

            // Draw logs
            let info_style = Style::default().fg(Color::Blue);
            let warning_style = Style::default().fg(Color::Yellow);
            let error_style = Style::default().fg(Color::Magenta);
            let critical_style = Style::default().fg(Color::Red);
            let logs: Vec<ListItem> = app
                .logs
                .items
                .iter()
                .map(|&(level, ref event)| {
                    let s = match level {
                        log::Level::Error => critical_style,
                        log::Level::Warn => warning_style,
                        _ => info_style,
                    };
                    let content = vec![text::Line::from(vec![
                        Span::styled(format!("{level:<9}"), s),
                        Span::raw(event),
                    ])];
                    ListItem::new(content)
                })
                .collect();
            let logs = List::new(logs).block(Block::bordered().title("List"));
            frame.render_stateful_widget(logs, chunks[1], &mut app.logs.state);
        }
        let bar_chart_grouped_temp_data: &Vec<(&str, u64)> = &app.barchart_temp.iter().map(|(x, y)| (x.as_str(), *y)).collect();
        let barchart = BarChart::default()
            .block(Block::bordered().title("Bar chart"))
            .data(bar_chart_grouped_temp_data)
            .bar_width(3)
            .bar_gap(2)
            .bar_set(if app.enhanced_graphics {
                symbols::bar::NINE_LEVELS
            } else {
                symbols::bar::THREE_LEVELS
            })
            .value_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Green)
                    .add_modifier(Modifier::ITALIC),
            )
            .label_style(Style::default().fg(Color::Yellow))
            .bar_style(Style::default().fg(Color::Green));
        frame.render_widget(barchart, chunks[1]);
    }
    if app.show_chart {
        let x_labels = vec![
            Span::styled(
                format!("{}", app.signals.window[0]),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(
                "{}",
                (app.signals.window[0] + app.signals.window[1]) / 2.0
            )),
            Span::styled(
                format!("{}", app.signals.window[1]),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ];
        let datasets = vec![
            Dataset::default()
                .name("data2")
                .marker(symbols::Marker::Dot)
                .style(Style::default().fg(Color::Cyan))
                .data(&app.signals.sin1.points),
            Dataset::default()
                .name("data3")
                .marker(if app.enhanced_graphics {
                    symbols::Marker::Braille
                } else {
                    symbols::Marker::Dot
                })
                .style(Style::default().fg(Color::Yellow))
                .data(&app.signals.sin2.points),
        ];
        let chart = Chart::new(datasets)
            .block(
                Block::bordered().title(Span::styled(
                    "Chart",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )),
            )
            .x_axis(
                Axis::default()
                    .title("X Axis")
                    .style(Style::default().fg(Color::Gray))
                    .bounds(app.signals.window)
                    .labels(x_labels),
            )
            .y_axis(
                Axis::default()
                    .title("Y Axis")
                    .style(Style::default().fg(Color::Gray))
                    .bounds([-20.0, 20.0])
                    .labels([
                        Span::styled("-20", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw("0"),
                        Span::styled("20", Style::default().add_modifier(Modifier::BOLD)),
                    ]),
            );
        frame.render_widget(chart, chunks[1]);
    }
}

fn draw_text(frame: &mut Frame, area: Rect) {
    let text = vec![
        text::Line::from("This is a paragraph with several lines. You can change style your text the way you want"),
        text::Line::from(""),
        text::Line::from(vec![
            Span::from("For example: "),
            Span::styled("under", Style::default().fg(Color::Red)),
            Span::raw(" "),
            Span::styled("the", Style::default().fg(Color::Green)),
            Span::raw(" "),
            Span::styled("rainbow", Style::default().fg(Color::Blue)),
            Span::raw("."),
        ]),
        text::Line::from(vec![
            Span::raw("Oh and if you didn't "),
            Span::styled("notice", Style::default().add_modifier(Modifier::ITALIC)),
            Span::raw(" you can "),
            Span::styled("automatically", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" "),
            Span::styled("wrap", Style::default().add_modifier(Modifier::REVERSED)),
            Span::raw(" your "),
            Span::styled("text", Style::default().add_modifier(Modifier::UNDERLINED)),
            Span::raw(".")
        ]),
        text::Line::from(
            "One more thing is that it should display unicode characters: 10€"
        ),
    ];
    let block = Block::bordered().title(Span::styled(
        "Footer",
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    ));
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn draw_second_tab(frame: &mut Frame, app: &mut App, area: Rect) {
    let chunks =
        Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)]).split(area);
    let up_style = Style::default().fg(Color::Green);
    let failure_style = Style::default()
        .fg(Color::Red)
        .add_modifier(Modifier::RAPID_BLINK | Modifier::CROSSED_OUT);
    let rows = app.servers.iter().map(|s| {
        let style = if s.status == "Up" {
            up_style
        } else {
            failure_style
        };
        Row::new(vec![s.name, s.location, s.status]).style(style)
    });
    let table = Table::new(
        rows,
        [
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Length(10),
        ],
    )
    .header(
        Row::new(vec!["Server", "Location", "Status"])
            .style(Style::default().fg(Color::Yellow))
            .bottom_margin(1),
    )
    .block(Block::bordered().title("Servers"));
    frame.render_widget(table, chunks[0]);

    let map = Canvas::default()
        .block(Block::bordered().title("World"))
        .paint(|ctx| {
            ctx.draw(&Map {
                color: Color::White,
                resolution: MapResolution::High,
            });
            ctx.layer();
            ctx.draw(&Rectangle {
                x: 0.0,
                y: 30.0,
                width: 10.0,
                height: 10.0,
                color: Color::Yellow,
            });
            ctx.draw(&Circle {
                x: app.servers[2].coords.1,
                y: app.servers[2].coords.0,
                radius: 10.0,
                color: Color::Green,
            });
            for (i, s1) in app.servers.iter().enumerate() {
                for s2 in &app.servers[i + 1..] {
                    ctx.draw(&canvas::Line {
                        x1: s1.coords.1,
                        y1: s1.coords.0,
                        y2: s2.coords.0,
                        x2: s2.coords.1,
                        color: Color::Yellow,
                    });
                }
            }
            for server in &app.servers {
                let color = if server.status == "Up" {
                    Color::Green
                } else {
                    Color::Red
                };
                ctx.print(
                    server.coords.1,
                    server.coords.0,
                    Span::styled("X", Style::default().fg(color)),
                );
            }
        })
        .marker(if app.enhanced_graphics {
            symbols::Marker::Braille
        } else {
            symbols::Marker::Dot
        })
        .x_bounds([-180.0, 180.0])
        .y_bounds([-90.0, 90.0]);
    frame.render_widget(map, chunks[1]);
}

fn draw_third_tab(frame: &mut Frame, _app: &mut App, area: Rect) {
    let chunks = Layout::horizontal([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)]).split(area);
    let colors = [
        Color::Reset,
        Color::Black,
        Color::Red,
        Color::Green,
        Color::Yellow,
        Color::Blue,
        Color::Magenta,
        Color::Cyan,
        Color::Gray,
        Color::DarkGray,
        Color::LightRed,
        Color::LightGreen,
        Color::LightYellow,
        Color::LightBlue,
        Color::LightMagenta,
        Color::LightCyan,
        Color::White,
    ];
    let items: Vec<Row> = colors
        .iter()
        .map(|c| {
            let cells = vec![
                Cell::from(Span::raw(format!("{c:?}: "))),
                Cell::from(Span::styled("Foreground", Style::default().fg(*c))),
                Cell::from(Span::styled("Background", Style::default().bg(*c))),
            ];
            Row::new(cells)
        })
        .collect();
    let table = Table::new(
        items,
        [
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ],
    )
    .block(Block::bordered().title("Colors"));
    frame.render_widget(table, chunks[0]);
}
