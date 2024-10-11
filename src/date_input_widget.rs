use chrono::{Days, Months, NaiveDate};
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::prelude::{Stylize, Widget};
use ratatui::style::{Color, Style};

#[derive(Debug, PartialEq, Clone)]
pub enum Selection {
    None,
    Day,
    Month,
    Year,
}

impl Selection {
    fn as_date_part(&self) -> &str {
        match self {
            Selection::Day => "%d",
            Selection::Month => "%m",
            Selection::Year => "%Y",
            _ => panic!("{:?} is not a date part", self)
        }
    }
}

#[derive(Clone)]
pub struct DateInputState {
    pub date: NaiveDate,
    pub selection: Selection,
}

#[derive(Clone)]
pub struct DateInputWidget {
    pub state: DateInputState,
    pub min: Option<NaiveDate>,
    pub max: Option<NaiveDate>,
}

impl Widget for &mut DateInputWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let default_style = Style::default().bold();
        let parts: Vec<(&str, Style)> = [Selection::Day, Selection::Month, Selection::Year].iter()
            .map(|sel| {
                let style = if self.state.selection == *sel {
                    default_style.fg(Color::Yellow)
                } else {
                    default_style
                };
                (sel.as_date_part(), style)
            })
            .collect();
        let mut position_x = area.left();
        for i in 0..parts.len() {
            let (str, style) = parts[i];
            buf.set_string(
                position_x,
                area.top(),
                self.state.date.format(str).to_string(),
                style,
            );
            position_x += str.len() as u16;
            if i < parts.len() - 1 {
                buf.set_string(
                    position_x,
                    area.top(),
                    ".",
                    default_style,
                );
                position_x += 1;
            }
        }
    }
}

impl DateInputWidget {
    pub fn handle_input(&mut self, key: KeyEvent) {
        let state = &mut self.state;
        if state.selection == Selection::None {
            state.selection = Selection::Day;
            return;
        }
        match key.code {
            KeyCode::Left => {
                match state.selection {
                    Selection::Day => { state.selection = Selection::None }
                    Selection::Month => { state.selection = Selection::Day }
                    Selection::Year => { state.selection = Selection::Month }
                    _ => {}
                }
            }
            KeyCode::Right => {
                match state.selection {
                    Selection::Day => { state.selection = Selection::Month }
                    Selection::Month => { state.selection = Selection::Year }
                    Selection::Year => { state.selection = Selection::None }
                    _ => {}
                }
            }
            KeyCode::Up => {
                match state.selection {
                    Selection::Day => { state.date = state.date.checked_add_days(Days::new(1)).unwrap() }
                    Selection::Month => { state.date = state.date.checked_add_months(Months::new(1)).unwrap() }
                    Selection::Year => { state.date = state.date.checked_add_months(Months::new(12)).unwrap() }
                    _ => {}
                }
                if self.max.is_some() && state.date > self.max.unwrap() {
                    state.date = self.max.unwrap()
                }
            }
            KeyCode::Down => {
                match state.selection {
                    Selection::Day => { state.date = state.date.checked_sub_days(Days::new(1)).unwrap() }
                    Selection::Month => { state.date = state.date.checked_sub_months(Months::new(1)).unwrap() }
                    Selection::Year => { state.date = state.date.checked_sub_months(Months::new(12)).unwrap() }
                    _ => {}
                }
                if self.min.is_some() && state.date < self.min.unwrap() {
                    state.date = self.min.unwrap()
                }
            }
            _ => {}
        };
    }
}