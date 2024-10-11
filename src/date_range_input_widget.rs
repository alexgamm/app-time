use crate::date_input_widget::{DateInputState, DateInputWidget, Selection};
use chrono::NaiveDate;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Offset, Rect};
use ratatui::prelude::Widget;
use ratatui::style::Style;

pub struct DateRangeInputWidget {
    pub min: Option<NaiveDate>,
    pub max: Option<NaiveDate>,
    inputs: (DateInputWidget, DateInputWidget),
}

impl Widget for &mut DateRangeInputWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (from, to) = &mut self.inputs;
        from.render(area, buf);
        buf.set_string(area.left() + 11, area.top(), "-", Style::default());
        to.render(area.offset(Offset { x: 13, y: 0 }), buf);
    }
}

impl DateRangeInputWidget {
    pub fn new(min: NaiveDate, max: NaiveDate) -> Self {
        Self {
            inputs: (
                DateInputWidget {
                    state: DateInputState { date: min, selection: Selection::None },
                    min: min.into(),
                    max: max.into(),
                },
                DateInputWidget {
                    state: DateInputState { date: max, selection: Selection::None },
                    min: min.into(),
                    max: max.into(),
                }
            ),
            min: min.into(),
            max: max.into(),
        }
    }
    pub fn is_any_selected(&self) -> bool {
        let (from, to) = &self.inputs;
        from.state.selection != Selection::None || to.state.selection != Selection::None
    }

    pub fn handle_input(&mut self, key: KeyEvent) {
        let is_any_selected = self.is_any_selected();
        let (from, to) = &mut self.inputs;
        if !is_any_selected {
            match key.code {
                KeyCode::Left => { to.state.selection = Selection::Year }
                KeyCode::Right => { from.state.selection = Selection::Day }
                _ => {}
            }
            return;
        }
        if from.state.selection != Selection::None {
            from.handle_input(key);
            if from.state.selection == Selection::None && key.code == KeyCode::Right {
                to.state.selection = Selection::Day;
            }
            to.min = from.state.date.into();
        } else if to.state.selection != Selection::None {
            to.handle_input(key);
            if to.state.selection == Selection::None && key.code == KeyCode::Left {
                from.state.selection = Selection::Year;
            }
            from.max = to.state.date.into();
        }
    }
    pub fn get_time_period(&self) -> (u32, u32) {
        let (from, to) = &self.inputs;
        (
            from.state.date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp() as u32,
            to.state.date.and_hms_opt(23, 59, 59).unwrap().and_utc().timestamp() as u32,
        )
    }
}