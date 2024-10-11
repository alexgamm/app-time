use chrono::Local;
use common::date_range_input_widget::DateRangeInputWidget;
use common::datetime::DateTimeExtensions;
use common::db::{Db, WindowStat};
use ratatui::crossterm::event;
use ratatui::crossterm::event::{KeyCode, KeyEventKind};
use ratatui::layout::{Direction, Layout};
use ratatui::prelude::{Constraint, Style};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Block, Cell, Padding, Row, Table, Tabs};
use ratatui::DefaultTerminal;
use std::io;
use std::time::Duration;

pub fn main() -> io::Result<()> {
    let db = Db::init().unwrap();
    let mut terminal = ratatui::init();
    terminal.clear()?;
    UI { terminal, db }.run()
}

pub struct UI {
    terminal: DefaultTerminal,
    db: Db,
}
impl UI {
    pub fn run(&mut self) -> io::Result<()> {
        let mut selected_tab = 0;
        let periods: Vec<StatsPeriod> = vec![
            StatsPeriod::Total,
            StatsPeriod::Today,
            StatsPeriod::Yesterday,
            StatsPeriod::Last3Days,
            StatsPeriod::ThisWeek,
            StatsPeriod::LastWeek,
            StatsPeriod::Custom
        ];
        let min_date = self.db.get_min_date().unwrap();
        let mut date_range_input = DateRangeInputWidget::new(
            min_date,
            Local::now().date_naive(),
        );
        loop {
            let is_custom = selected_tab == periods.len() - 1;

            let titles = periods.iter()
                .map(|period| {
                    if *period == StatsPeriod::Custom
                        && is_custom { "" } else { period.as_title() }
                })
                .collect::<Vec<&str>>();
            let paddings = (" ", " ");
            let divider = "|";
            let titles_width = titles.iter()
                .map(|title| title.len() + paddings.0.len() + paddings.1.len())
                .sum::<usize>() + titles.len() * divider.len() - 2;
            self.terminal.draw(|frame| {
                let tabs = Tabs::new(
                    titles.into_iter()
                        .map(Line::from)
                        .collect::<Vec<Line>>()
                )
                    .style(Style::default().white())
                    .highlight_style(Style::default().on_yellow().black())
                    .divider(divider)
                    .padding(paddings.0, paddings.1)
                    .select(selected_tab);

                let table_block = Block::default()
                    .padding(Padding::left(1));
                let time_period = if is_custom {
                    date_range_input.get_time_period().into()
                } else {
                    periods[selected_tab].as_time_period()
                };
                let rows: Vec<WindowStatRow> = self.db.get_stats(time_period)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|window_stat| { WindowStatRow { window_stat } })
                    .collect();

                let total_seconds = rows.iter().clone()
                    .map(|row| row.window_stat.seconds)
                    .sum();

                let table = Table::new(
                    rows.iter().map(|row| row.create_row(total_seconds)),
                    [
                        Constraint::Length(30),
                        Constraint::Length(1),
                        Constraint::Length(30),
                        Constraint::Length(1),
                        Constraint::Length(4),
                        Constraint::Length(1),
                        Constraint::Length(20),
                    ],
                ).block(table_block);

                let layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(vec![
                        Constraint::Length(2),
                        Constraint::Fill(1),
                    ])
                    .split(frame.area());
                if is_custom {
                    let tabs_layout = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(vec![
                            Constraint::Length(titles_width as u16),
                            Constraint::Fill(1),
                        ])
                        .split(layout[0]);
                    frame.render_widget(&tabs, tabs_layout[0]);
                    frame.render_widget(&mut date_range_input, tabs_layout[1]);
                } else {
                    frame.render_widget(&tabs, layout[0]);
                }
                frame.render_widget(&table, layout[1]);
            })?;

            if event::poll(Duration::from_secs(5))? {
                if let event::Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        if date_range_input.is_any_selected() {
                            date_range_input.handle_input(key);
                            if !date_range_input.is_any_selected() {
                                match key.code {
                                    KeyCode::Left => selected_tab = periods.len() - 2,
                                    KeyCode::Right => selected_tab = 0,
                                    _ => {}
                                };
                            }
                        } else {
                            match key.code {
                                KeyCode::Left => selected_tab = (selected_tab + periods.len() - 1) % periods.len(),
                                KeyCode::Right => selected_tab = (selected_tab + 1) % periods.len(),
                                _ => {}
                            };
                            if selected_tab == periods.len() - 1 {
                                date_range_input.handle_input(key);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(PartialEq)]
enum StatsPeriod {
    Total,
    Today,
    Yesterday,
    Last3Days,
    ThisWeek,
    LastWeek,
    Custom,
}

impl StatsPeriod {
    fn as_title(&self) -> &str {
        match self {
            StatsPeriod::Total => "Total",
            StatsPeriod::Today => "Today",
            StatsPeriod::Yesterday => "Yesterday",
            StatsPeriod::Last3Days => "Last 3 days",
            StatsPeriod::ThisWeek => "This week",
            StatsPeriod::LastWeek => "Last week",
            StatsPeriod::Custom => "Custom",
        }
    }
    fn as_time_period(&self) -> Option<(u32, u32)> {
        let now = Local::now();
        let now_ts = now.timestamp() as u32;
        match self {
            StatsPeriod::Today => Some((
                now.start_of_day_ts(0),
                now_ts
            )),
            StatsPeriod::Yesterday => Some((
                now.start_of_day_ts(1),
                now.start_of_day_ts(0)
            )),
            StatsPeriod::Last3Days => Some((
                now.start_of_day_ts(2),
                now_ts
            )),
            StatsPeriod::ThisWeek => Some((
                now.start_of_week_ts(0),
                now_ts
            )),
            StatsPeriod::LastWeek => Some((
                now.start_of_week_ts(1),
                now.start_of_week_ts(0)
            )),
            _ => None,
        }
    }
}

struct WindowStatRow {
    window_stat: WindowStat,
}

impl WindowStatRow {
    fn create_row(&self, total_window_stats_seconds: u32) -> Row {
        let window_name = String::from(&self.window_stat.window_name);
        let window_stat_time = Self::format_time(self.window_stat.seconds);
        let ratio = self.window_stat.seconds as f64 / total_window_stats_seconds as f64;
        let percentage = (ratio * 100.0).round();
        let progress_bar = Self::progress_bar(30, ratio);
        Row::new(vec![
            Cell::from(window_name),
            Cell::from("│"),
            Cell::from(progress_bar).yellow(),
            Cell::from("│"),
            Cell::from(format!("{percentage}%")),
            Cell::from("│"),
            Cell::from(window_stat_time)
        ])
    }

    fn format_time(seconds: u32) -> String { // TODO REFACTOR ASAP
        let minutes = seconds / 60;
        if minutes == 0 {
            return format!("{seconds}s");
        }
        let hours = minutes / 60;
        if hours == 0 {
            return format!("{minutes}m {}s", seconds % 60);
        }
        let days = hours / 24;
        if days == 0 {
            return format!("{hours}h {}m {}s", minutes % 60, seconds % 60);
        }
        format!("{days}d {}h {}m {}s", hours % 24, minutes % 60, seconds % 60)
    }

    fn progress_bar(length: u16, ratio: f64) -> String {
        let progress = (length as f64 * ratio).floor() as usize;
        vec!["▀"; progress].join("")
    }
}
