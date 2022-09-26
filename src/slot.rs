use std::time::{Duration, SystemTime};

pub enum Slots {
    One(u32),
    Two(u32),
}

pub struct Slot {
    slot_1: u32,
    slot_2: u32,
    slot_1_time: SystemTime,
    slot_2_time: SystemTime,
}

impl Slot {
    pub fn check(slot: Slots) -> bool {
        match slot {
            Slots::One(stream) => {}
            Slots::Two(stream) => {}
        }
        false
    }

    pub fn init() -> Self {
        let t = SystemTime::now();
        Self {
            slot_1: 0,
            slot_2: 0,
            slot_1_time: t,
            slot_2_time: t,
        }
    }

    pub fn lock(&mut self, slot: Slots) -> bool {
        match slot {
            Slots::One(stream) => {
                if self.slot_1 == stream {
                    return true;
                } else if self.slot_1 != 0 && !self.unlock(slot){
                    return false;
                }

                self.slot_1 = stream;
                self.slot_1_time = SystemTime::now()
            }
            Slots::Two(stream) => {
                if self.slot_2 == stream {
                    return true;
                } else if self.slot_2 != 0 && !self.unlock(slot){
                    return false;
                }

                self.slot_2 = stream;
                self.slot_1_time = SystemTime::now()
            }
        }
        true
    }

    fn unlock(&mut self, slot: Slots) -> bool {
        match slot {
            Slots::One(_) => {
                if let Ok(elp) = self.slot_1_time.elapsed(){
                    if elp.as_millis() > 256 {
                        println!("Slot 1 unlocked");
                        return true
                    }
                }
            }
            Slots::Two(_) => {
                if let Ok(elp) = self.slot_2_time.elapsed(){
                    if elp.as_millis() > 256 {
                        println!("Slot 2 unlocked");
                        return true
                    }
                }
            }
        }
        false
    }
}
