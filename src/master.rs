use std::time::SystemTime;

pub struct State{
    talkgroup: u32,
    sl: u32,
    stream: u32,
    timestamp: SystemTime,
}

impl State{
    fn default() -> Self {
        Self {
            talkgroup: 0,
            sl: 1,
            stream: 0,
            timestamp: std::time::SystemTime::now(),
        }
    }
}

pub fn test_master() {
    println!("testing");
}