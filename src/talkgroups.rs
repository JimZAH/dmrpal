use crate::peers::Peertype;
use std::time::SystemTime;

pub enum TgActivate {
    Static(u32),
    Ua(u32),
}

#[derive(Debug)]
pub struct Talkgroup {
    pub expire: u64,
    pub id: u32,
    pub la: SystemTime,
    pub routeable: Peertype,
    pub sl: u8,
    pub ua: bool,
    pub time_stamp: SystemTime,
}

impl Talkgroup {
    // return a default value for talkgroup
    pub fn default() -> Self {
        Self {
            expire: 0,
            id: 0,
            la: SystemTime::now(),
            routeable: Peertype::Local,
            sl: 1,
            ua: false,
            time_stamp: SystemTime::now(),
        }
    }

    // Remove a talkgroup from a peer
    pub fn ua_clear(&mut self) -> bool {
        if self.ua {
            return match self.time_stamp.elapsed() {
                Ok(ts) => {
                    if ts.as_secs() > self.expire {
                        // If the talkgroup has traffic, skip and try again when there's no traffic
                        if let Ok(la) = self.la.elapsed() {
                            if la.as_secs() <= 5 {
                                return true;
                            }
                        };
                        println!("Removing TG: {}, From Peer: {}", self.id, self.id);
                        false
                    } else {
                        true
                    }
                }
                Err(_) => {
                    println!("There was an error passing time for UA, removing TG!");
                    false
                }
            };
        }
        true
    }

    // Set a talk group to a peer
    pub fn set(sl: u8, tg: TgActivate, exp: Option<u64>) -> Self {
        let (ua, talk_group, expire) = match tg {
            TgActivate::Static(u) => (false, u, 0),
            TgActivate::Ua(u) => {
                let e: u64 = match exp {
                    Some(v) => v,
                    None => 900,
                };
                (true, u, e)
            }
        };

        Self {
            expire,
            id: talk_group,
            la: SystemTime::now(),
            routeable: Peertype::Local,
            sl,
            ua,
            time_stamp: SystemTime::now(),
        }
    }
}
