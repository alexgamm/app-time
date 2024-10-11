use chrono::{DateTime, Local, NaiveDate};
use rusqlite::Connection;
use std::error::Error;
use std::path::PathBuf;
use std::{env, fs};


pub struct Db {
    connection: Connection,
}

impl Db {
    pub fn get_path() -> PathBuf {
        let local_data_dir = env::var("LOCALAPPDATA").unwrap_or_else(|_| {
            panic!("Could not find env LOCALAPPDATA in the windows system");
        });
        let mut db_path = PathBuf::from(local_data_dir);
        db_path.push("app-time");

        if !db_path.exists() {
            fs::create_dir_all(&db_path).expect("Could not create a directory in AppData\\Local");
        }

        db_path.push("db.sqlite");
        db_path
    }

    pub fn init() -> Result<Db, Box<dyn Error>> {
        let db = Db {
            connection: Connection::open(Self::get_path())?
        };
        db.connection.execute("create table if not exists activity (
            window_name text not null,
            time_from integer not null,
            time_to integer not null
        )", ())?;
        Ok(db)
    }

    pub fn update_last(&self, window_name: &String, time_to: u32) -> Result<(), Box<dyn Error>> {
        self.connection.execute("update activity 
            set time_to = ?
            where rowid = (
                select rowid from activity 
                where window_name = ? 
                order by time_from desc 
                limit 1
        )", (time_to, window_name))?;
        Ok(())
    }

    pub fn insert(&self, window_name: &String, time_from: u32) -> Result<(), Box<dyn Error>> {
        self.connection.execute(
            "insert into activity (window_name, time_from, time_to) 
                 values (?1, ?2, ?2)",
            (&window_name, time_from),
        )?;
        Ok(())
    }

    pub fn get_stats(&self, period: Option<(u32, u32)>) -> Result<Vec<WindowStat>, Box<dyn Error>> {
        let condition = period.map(|(from, to)| {
            format!("and time_from >= {from} and time_to <= {to}")
        }).unwrap_or_default();
        let mut statement = self.connection.prepare(&format!(
            "select window_name, sum(time_to - time_from) as time
                  from activity
                  where time > 0 {condition}
                  group by window_name
                  order by time desc"
        ))?;
        let result = statement.query_map([], |row| {
            Ok(
                WindowStat {
                    window_name: row.get(0)?,
                    seconds: row.get(1)?,
                }
            )
        })?
            .map(|row| { row.unwrap() }) // TODO deal with panic
            .collect();
        Ok(result)
    }
    pub fn get_min_date(&self) -> Result<NaiveDate, Box<dyn Error>> {
        let mut statement = self.connection.prepare(
            "select min(time_from) from activity"
        )?;
        let default_min_date = Local::now().date_naive();
        let result = statement.query_map([], |row| {
            row.get::<_, i64>(0)
                .map(|min_ts| DateTime::from_timestamp(min_ts, 0).unwrap())
                .map(|date_time| date_time.date_naive())
        })?
            .map(|row| { row.unwrap_or(default_min_date) })
            .next()
            .unwrap_or(default_min_date);
        Ok(result)
    }
}

pub struct WindowStat {
    pub window_name: String,
    pub seconds: u32,
}

