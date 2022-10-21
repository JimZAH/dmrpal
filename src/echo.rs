use std::time::SystemTime;

// Frame struct that hold data to echo
pub struct Frame {
    pub data: [u8; 55],
    stream: u32,
}

pub struct Queue {
    pub echos: Vec<Frame>,
    pub la_time: SystemTime,
}

impl Frame {
    pub fn create(data: [u8; 55], stream: u32) -> Self {
        Self { data, stream }
    }

    pub fn commit(self, q: &mut Queue) {
        Queue::submit(q, self)
    }
}

impl Queue {
    pub fn default() -> Self {
        Self {
            echos: Vec::new(),
            la_time: SystemTime::now(),
        }
    }

    pub fn has_items(&self) -> bool {
        self.echos.is_empty()
    }

    pub fn submit(&mut self, frame: Frame) {
        println!("Submitting to Queue for stream ID: {}", frame.stream);
        self.echos.push(frame);
        self.la_time = SystemTime::now();
    }
}
