use std::{thread, time};

pub mod echo;
pub mod master;
pub mod peers;
pub mod slot;
pub mod streams;
pub mod system;
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

/* Log printing macro */
#[macro_export]
macro_rules! dprint {
    ($verbose:expr;$mid:expr;$($arg:tt)*) => {{
    if $verbose >= $mid {
        match $mid{
            1 => print!("CRITICAL: "),
            2 => print!("WARNING: "),
            3 => print!("NOTICE: "),
            4 => print!("INFO: "),
            10 => print!("DEBUG: "),
            _=> {},
        };
        println!($($arg)*);
    }
    }};
}

/* Pause for X microseconds */
pub fn sleep(time: u64) {
    thread::sleep(time::Duration::from_micros(time));
}
