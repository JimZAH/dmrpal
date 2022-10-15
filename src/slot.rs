use std::time::SystemTime;

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
        let t = SystemTime::now();
        match slot {
            Slots::One(tg) => {
                if self.slot_1 == tg {
                    self.slot_1_time = t;
                    return true;
                } else if !self.unlock(slot) {
                    return false;
                }

                self.slot_1 = tg;
            }
            Slots::Two(tg) => {
                if self.slot_2 == tg {
                    self.slot_2_time = t;
                    return true;
                } else if !self.unlock(slot) {
                    return false;
                }

                self.slot_2 = tg;
            }
        }
        true
    }

    fn unlock(&mut self, slot: Slots) -> bool {
        match slot {
            Slots::One(_) => {
                if let Ok(elp) = self.slot_1_time.elapsed() {
                    if elp.as_secs() > 5 {
                        println!("Slot 1 unlocked");
                        return true;
                    }
                }
            }
            Slots::Two(_) => {
                if let Ok(elp) = self.slot_2_time.elapsed() {
                    if elp.as_secs() > 5 {
                        println!("Slot 2 unlocked");
                        return true;
                    }
                }
            }
        }
        false
    }
}
