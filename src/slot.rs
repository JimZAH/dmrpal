pub enum Slots {
    One(u32),
    Two(u32),
}

pub struct Slot {
    duplex: u8,
    slot_1: u32,
    slot_2: u32,
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
        Self {
            duplex: 1,
            slot_1: 0,
            slot_2: 0,
        }
    }

    pub fn lock(&mut self, slot: Slots) -> bool {
        match slot {
            Slots::One(stream) => {
                if self.slot_1 == stream {
                    return true;
                } else if self.slot_1 != 0 {
                    return false;
                }

                self.slot_1 = stream
            }
            Slots::Two(stream) => {
                if self.slot_2 == stream {
                    return true;
                } else if self.slot_2 != 0 {
                    return false;
                }

                self.slot_2 = stream
            }
        }
        true
    }

    pub fn unlock(&mut self, slot: Slots) -> bool {
        match slot {
            Slots::One(stream) => {
                if self.slot_1 == stream {
                    self.slot_1 = 0;
                    return true;
                }
            }
            Slots::Two(stream) => {
                if self.slot_2 == stream {
                    self.slot_2 = 0;
                    return true;
                }
            }
        }
        false
    }
}
