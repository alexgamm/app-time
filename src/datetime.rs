use chrono::{DateTime, Datelike, Local, TimeDelta, Timelike};

pub trait DateTimeExtensions {
    fn start_of_day(&self) -> Option<DateTime<Local>>;
    fn start_of_day_ts(&self, days_to_subtract: i64) -> u32;
    fn start_of_week_ts(&self, weeks_to_subtract: i64) -> u32;
    fn num_days_between_starts(&self, to: DateTime<Local>) -> i64;
}

impl DateTimeExtensions for DateTime<Local> {
    fn start_of_day(&self) -> Option<DateTime<Local>> {
        self.with_hour(0)
            .and_then(|dt| dt.with_minute(0))
            .and_then(|dt| dt.with_second(0))
            .and_then(|dt| dt.with_nanosecond(0))
    }
    fn start_of_day_ts(&self, days_to_subtract: i64) -> u32 {
        (self.clone() - TimeDelta::days(days_to_subtract))
            .start_of_day()
            .unwrap()
            .timestamp() as u32
    }
    fn start_of_week_ts(&self, weeks_to_subtract: i64) -> u32 {
        (self.clone() - TimeDelta::weeks(weeks_to_subtract))
            .start_of_day_ts(self.weekday().num_days_from_monday() as i64)
    }
    fn num_days_between_starts(&self, to: DateTime<Local>) -> i64 {
        (to.start_of_day().unwrap() - self.start_of_day().unwrap()).num_days()
    }
}