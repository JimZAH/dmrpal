use std::{thread, time};

pub mod echo;
pub mod master;
pub mod peers;
pub mod slot;
pub mod streams;
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

#[macro_export]
macro_rules! dprint {
    ($verbose:expr;$mid:expr;$($arg:tt)*) => {{
    if $verbose >= $mid {
        let mut prefix = "";
        match $verbose{
            1 => print!("CRITICAL: "),
            2 => print!("WARNING: "),
            3 => print!("NOTICE: "),
            4 => print!("INFO: "),
            _=> {
                print!("DEBUG: ");
            }
        };
        println!($($arg)*);
    }
    }};
}

// Pause for X microseconds
pub fn sleep(time: u64) {
    thread::sleep(time::Duration::from_micros(time));
}
