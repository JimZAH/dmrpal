use std::time;

/* system.rs
    Data related to the DMRPal System.
*/

pub struct System {
    pub master_reconnects: usize,
    pub total_timeouts: usize,
    pub uptime: time::SystemTime,
}

impl System {
    pub fn init() -> Self {
        Self {
            master_reconnects: 0,
            total_timeouts: 0,
            uptime: time::SystemTime::now(),
        }
    }
}
