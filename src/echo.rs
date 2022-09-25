use std::{net::SocketAddr, time::SystemTime};

// Frame struct that hold data to echo
pub struct Frame {
    data: [u8; 55],
    stream: u32,
}

#[derive(Default)]
pub struct Queue {
    echos: Vec<Frame>,
}

impl Frame {
    pub fn create(data: [u8; 55], src: SocketAddr, stream: u32) -> Self {
        Self {
            data,
            stream,
        }
    }

    pub fn commit(self, q: &mut Queue) {
        Queue::push(q, self)
    }
}

impl Queue {
    fn push(&mut self, frame: Frame) {
        self.echos.push(frame)
    }
}
