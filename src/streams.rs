use std::{collections::hash_map::HashMap, time::SystemTime};

/* streams.rs
    Store DMR stream ID data, this can be used for timeouts and stats.
*/

// Store stream data, maybe we also store talk group info in future
#[derive(Eq, Hash, PartialEq)]
pub struct Stream {
    pub id: u32,
    pub end_time: SystemTime,
    pub start_time: SystemTime,
    pub time_out: bool,
}

pub struct Streams {
    pub current_streams: HashMap<u32, Stream>,
    pub total: usize,
}

impl Stream {
    fn start(id: u32) -> Self {
        Self {
            id,
            end_time: SystemTime::now(),
            start_time: SystemTime::now(),
            time_out: false,
        }
    }

    fn update_end(&mut self) {
        self.end_time = SystemTime::now();
    }
}

impl Streams {
    pub fn init() -> Self {
        Self {
            current_streams: HashMap::new(),
            total: 0,
        }
    }

    pub fn stream(&mut self, id: u32) {
        if let Some(v) = self.current_streams.get_mut(&id) {
            Stream::update_end(v);
            return;
        }

        self.total += 1;
        self.current_streams.insert(id, Stream::start(id));
    }

    // Check if we have any redundant streams
    pub fn check(&mut self) {
        self.current_streams
            .retain(|_, v| match v.end_time.elapsed() {
                Ok(e) => {
                    if e.as_secs() >= 5 {
                        return false;
                    } else {
                        return true;
                    }
                }
                Err(_) => false,
            })
    }
}
