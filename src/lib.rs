use std::{dbg, thread, time};

pub enum SystemState {
    Idle,
    Inuse(u32),
}

pub struct Systemstate {
    pub state: SystemState,
    pub time: time::SystemTime,
}

pub fn debug(text: &str) {
    dbg!(text);
}

pub fn sleep(time: u64) {
    thread::sleep(time::Duration::from_micros(time));
}
