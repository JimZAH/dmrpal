use std::{dbg, thread, time};

pub mod echo;
pub mod master;
pub mod peers;
pub mod slot;
pub mod talkgroups;

// Not yet used
pub enum SystemState {
    Idle,
    Inuse(u32),
}

// Not yet used
pub struct Systemstate {
    pub state: SystemState,
    pub time: time::SystemTime,
}

// Convert utf8 to u32
pub fn utint(num: &str) -> u32 {
    match num.parse::<u32>() {
        Ok(v) => v,
        Err(_) => 0,
    }
}

// Will need a better debug approach but this works for now
pub fn debug(text: &str) {
    dbg!(text);
}

// Pause for X microseconds
pub fn sleep(time: u64) {
    thread::sleep(time::Duration::from_micros(time));
}
