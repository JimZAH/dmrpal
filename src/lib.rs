use std::{dbg, thread, time};

pub fn debug(text: &str) {
    dbg!(text);
}

pub fn sleep(time: u64) {
    thread::sleep(time::Duration::from_millis(time));
}
