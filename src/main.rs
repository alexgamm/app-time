use crate::db::Db;
use crate::tracker::Tracker;

mod window;
mod db;
mod tracker;

fn main() {
    let tracker = Tracker {
        db: Db::init().unwrap()
    };
    tracker.init()
}

