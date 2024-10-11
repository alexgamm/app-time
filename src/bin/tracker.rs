#![windows_subsystem = "windows"]

use auto_launch::AutoLaunchBuilder;
use chrono::{Local, TimeDelta};
use common::datetime::DateTimeExtensions;
use common::db::Db;
use common::tray::{Events, Tray};
use common::window::Window;
use std::env::current_exe;
use std::process::{Child, Command};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let db_exists = Db::get_path().exists();
    if !db_exists{
        enable_auto_launch();
    }
    thread::spawn(move || {
        let tracker = Tracker {
            db: Db::init().unwrap()
        };
        tracker.init();
    });
    let (s, r) = std::sync::mpsc::channel::<Events>();
    let mut tray = Tray::init(s);
    thread::spawn(move || {
        let mut ui: Option<Child> = None;
        if !db_exists {
            ui = spawn_ui();
        }
        r.iter().for_each(|m| {
            match m {
                Events::RightClickTrayIcon => {
                    tray.show_menu();
                }
                Events::DoubleClickTrayIcon | Events::Open => {
                    if let Some(child) = &mut ui {
                        child.kill().unwrap();
                    }
                    ui = Command::new("ui.exe").spawn().unwrap().into();
                }
                Events::Exit => {
                    if let Some(child) = &mut ui {
                        child.kill().unwrap();
                    }
                    std::process::exit(0);
                }
            }
        })
    });
    Tray::handle_win_messages();
}

fn enable_auto_launch() -> bool {
    let current_exe = current_exe().ok()
        .and_then(|path| path.to_str().map(|s| s.to_string()));
    if current_exe.is_none() {
        return false;
    }
    AutoLaunchBuilder::new()
        .set_app_name("AppTime")
        .set_app_path(&current_exe.unwrap())
        .build()
        .and_then(|auto| auto.enable())
        .ok()
        .is_some()
}

fn spawn_ui() -> Option<Child> {
    Command::new("ui.exe").spawn().unwrap().into()
}

pub struct Tracker {
    pub db: Db,
}

impl Tracker {
    pub fn init(&self) {
        // был раб.стол - пришел раб.стол +
        // был раб.стол - пришло окно (update time_to) +
        // было окно - пришло такое же окно (update time_to) +
        // было окно - пришло окно +
        // было окно - пришел раб.стол +

        let mut display_name: String = String::new();
        let mut time_from = Local::now();
        loop {
            let window = Window::get_active();
            let new_display_name = window.get_display_name().unwrap();
            if display_name.is_empty() && new_display_name.is_empty() {
                Self::sleep();
                continue;
            }
            let now = Local::now().timestamp() as u32;
            if !display_name.is_empty() {
                for i in 0..time_from.num_days_between_starts(Local::now()) {
                    let new_day = (time_from + TimeDelta::days(i + 1))
                        .start_of_day()
                        .unwrap()
                        .timestamp() as u32;
                    self.db.update_last(&display_name.clone(), new_day).unwrap();
                    self.db.insert(&new_display_name.clone(), new_day).unwrap();
                }
                self.db.update_last(&display_name.clone(), now).unwrap();
            }
            if !display_name.eq(&new_display_name) && !new_display_name.is_empty() {
                self.db.insert(&new_display_name.clone(), now).unwrap();
                time_from = Local::now();
            }
            display_name = new_display_name;
            Self::sleep();
        }
    }

    fn sleep() {
        sleep(Duration::from_secs(5))
    }
}
