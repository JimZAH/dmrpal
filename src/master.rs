use std::time::SystemTime;

pub struct State{
    talkgroup: u32,
    timestamp: SystemTime,
}

impl State{
    fn default() -> Self {
        Self {
            talkgroup: 0,
            timestamp: std::time::SystemTime::now(),
        }
    }
}

pub fn test_master() {
    println!("testing");
}