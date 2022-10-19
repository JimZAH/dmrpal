use crate::{
    echo::{self, Queue},
    slot,
    talkgroups::{Talkgroup, TgActivate},
};
use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr},
    string,
    time::SystemTime,
};

#[derive(Debug, PartialEq)]
pub enum Peertype {
    Local,
    Friend,
    All,
}

pub struct Peer {
    pub id: u32,
    pub callsign: String,
    pub duplex: u8,
    pub echo: echo::Queue,
    pub frequency: String,
    pub software: String,
    pub latitude: f32,
    pub last_check: SystemTime,
    pub longitude: f32,
    pub power: u16,
    pub height: u16,
    pub ip: std::net::SocketAddr,
    pub talk_groups: HashMap<u32, Talkgroup>,
    pub tx_bytes: usize,
    pub options: String,
    pub peer_type: Peertype,
    pub rx_bytes: usize,
    pub slot: slot::Slot,
    pub tg_expire: u64,
}

impl Peer {
    pub fn new() -> Self {
        Self {
            id: 0,
            callsign: string::String::default(),
            duplex: 0,
            echo: echo::Queue::default(),
            frequency: string::String::default(),
            software: string::String::default(),
            latitude: 0.0,
            last_check: SystemTime::now(),
            longitude: 0.0,
            power: 0,
            height: 0,
            ip: std::net::SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
            talk_groups: HashMap::from([
                (0, Talkgroup::default()),
                (31337, Talkgroup::set(2, TgActivate::Static(31337), None)),
                (2351, Talkgroup::set(1, TgActivate::Static(2351), None)),
                (235, Talkgroup::set(1, TgActivate::Static(235), None)),
                (844, Talkgroup::set(2, TgActivate::Static(844), None)),
                (840, Talkgroup::set(2, TgActivate::Static(840), None)),
                (123, Talkgroup::set(1, TgActivate::Static(123), None)),
                (113, Talkgroup::set(1, TgActivate::Static(113), None)),
                (3, Talkgroup::set(1, TgActivate::Static(3), None)),
                (2, Talkgroup::set(1, TgActivate::Static(2), None)),
                (1, Talkgroup::set(1, TgActivate::Static(1), None)),
            ]),
            tx_bytes: 0,
            options: string::String::default(),
            peer_type: Peertype::Local,
            rx_bytes: 0,
            slot: slot::Slot::init(),
            tg_expire: 0,
        }
    }

    // Check if the peer is allowed to sign in.
    pub fn acl(&self) -> bool {
        let known_peers = vec![000000];
        for k in known_peers {
            if self.id.eq(&k) {
                return false;
            }
        }
        true
    }

    pub fn echo(&mut self, data: [u8; 55], stream: u32) {
        let frame = echo::Frame::create(data,stream);
        self.echo.submit(frame);
    }

    pub fn options(&mut self) {
        for opts in self.options.split(';') {
            if opts.len() < 3 {
                continue;
            }
            match &opts[..3] {
                "TS1" | "TS2" => match opts.chars().nth(2) {
                    Some(s) => {
                        let slot = s as u8 - 48;
                        let mut tg: u32 = 0;
                        for i in opts[5..].bytes() {
                            if i > 47 && i < 58 {
                                tg = tg * 10;
                                tg = tg + i as u32 - 48;
                            }
                        }
                        self.talk_groups
                            .insert(tg, Talkgroup::set(slot, TgActivate::Static(tg), None));
                        println!("OPTIONS {}, Added {} {}", self.id, slot, tg);
                    }
                    None => continue,
                },
                "UAT" => {
                    let mut t: u64 = 0;
                    for i in opts[4..].bytes() {
                        if i > 47 && i < 58 {
                            t = t * 10;
                            t = t + i as u64 - 48;
                        }
                    }
                    self.tg_expire = t;
                }
                _ => continue,
            }
        }
    }

    // Set the peer ID
    pub fn pid(&mut self, buff: &[u8; 4]) {
        self.id = ((buff[0] as u32) << 24)
            | ((buff[1] as u32) << 16)
            | ((buff[2] as u32) << 8)
            | (buff[3] as u32);
    }
}
