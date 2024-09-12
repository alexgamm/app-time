use std::{env, fs};
use std::error::Error;
use std::path::PathBuf;

use rusqlite::Connection;

pub struct Db {
    connection: Connection,
}

impl Db {
    fn get_path() -> PathBuf {
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
            time_from integer,
            time_to integer
        )", ())?;
        Ok(db)
    }

    pub fn add(&self, window_name: &String) -> Result<(), Box<dyn Error>> {
        self.connection.execute("update activity 
            set time_to = strftime('%s', 'now')
            where rowid = (
                select rowid from activity 
                where time_to is null 
                order by time_from desc 
                limit 1
        )", ())?;
        self.connection.execute(
            "insert into activity (window_name, time_from) 
                 values (?1, strftime('%s', 'now'))",
            (&window_name, ),
        )?;
        Ok(())
    }
}
