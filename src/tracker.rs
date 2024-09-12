use std::thread::sleep;
use std::time::Duration;
use crate::db::Db;
use crate::window::Window;

pub struct Tracker {
   pub db: Db,
}

impl Tracker {
    pub fn init(&self) {
        let mut display_name = String::new();
        loop {
            let window = Window::get_active();
            let current = window.get_display_name();
            if current.is_none() {
                // TODO log
                Self::sleep();
                continue;
            }
            let unwrapped_current = current.unwrap();
            if display_name.eq(&unwrapped_current) {
                Self::sleep();
                continue;
            }
            display_name = unwrapped_current;
            println!("{display_name}");
            self.db.add(&display_name).map_err(|err| {
                //TODO log
            }).unwrap();
            Self::sleep();
        }
    }

    fn sleep() {
        sleep(Duration::from_secs(5))
    }
}