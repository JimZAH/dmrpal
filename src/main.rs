use std::collections::{hash_map::HashMap, hash_set::HashSet};
use std::net::{IpAddr, Ipv4Addr, UdpSocket};
use std::{io, str, string, time::SystemTime};

mod db;
mod hb;

const SOFTWARE_VERSION: u64 = 1;

#[derive(Debug, PartialEq)]
enum Peertype {
    Local,
    Friend,
    All,
}

enum TgActivate {
    Static(u32),
    Ua(u32),
}

struct Peer {
    id: u32,
    Callsign: String,
    Duplex: u8,
    Frequency: String,
    Software: String,
    Latitude: f32,
    last_check: SystemTime,
    Longitude: f32,
    Power: u16,
    Height: u16,
    ip: std::net::SocketAddr,
    talk_groups: HashMap<u32, Talkgroup>,
    peer_type: Peertype,
}

#[derive(Debug)]
struct Talkgroup {
    expire: u64,
    id: u32,
    routeable: Peertype,
    sl: u8,
    ua: bool,
    time_stamp: SystemTime,
}

impl Peer {
    fn new() -> Self {
        Self {
            id: 0,
            Callsign: string::String::default(),
            Duplex: 0,
            Frequency: string::String::default(),
            Software: string::String::default(),
            Latitude: 0.0,
            last_check: SystemTime::now(),
            Longitude: 0.0,
            Power: 0,
            Height: 0,
            ip: std::net::SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
            talk_groups: HashMap::from([
                (0, Talkgroup::default()),
                (31337, Talkgroup::set(2, TgActivate::Static(31337))),
            ]),
            peer_type: Peertype::Local,
        }
    }

    // Check if the peer is allowed to sign in.
    fn acl(&self) -> bool {
        let known_peers = vec![000000];
        for k in known_peers {
            if self.id.eq(&k) {
                return false;
            }
        }
        true
    }

    // Set the peer ID
    fn pid(&mut self, buff: &[u8; 4]) {
        self.id = ((buff[0] as u32) << 24)
            | ((buff[1] as u32) << 16)
            | ((buff[2] as u32) << 8)
            | (buff[3] as u32);
    }
}

impl Talkgroup {
    // return a default value for talkgroup
    fn default() -> Self {
        Self {
            expire: 0,
            id: 0,
            routeable: Peertype::Local,
            sl: 1,
            ua: false,
            time_stamp: SystemTime::now(),
        }
    }

    // Remove a talkgroup from a peer
    fn remove(&mut self, tg: u32) -> bool {
        false
    }

    // Set a talk group to a peer
    fn set(sl: u8, tg: TgActivate) -> Self {
        let (ua, talk_group, exp) = match tg {
            TgActivate::Static(u) => (false, u, 0),
            TgActivate::Ua(u) => (true, u, 900),
        };

        Self {
            expire: exp,
            id: talk_group,
            routeable: Peertype::All,
            sl: sl,
            ua: ua,
            time_stamp: SystemTime::now(),
        }
    }
}

// If we've not heard from a peer in a while remove them
fn clock(logins: &mut HashSet<u32>, mash: &mut HashMap<u32, Peer>) {
    //TODO
}

fn echo(sock: &std::net::UdpSocket, dst: std::net::SocketAddr, data: &Vec<[u8; 55]>) {
    for d in data {
        sock.send_to(d, dst).unwrap();
    }
}

// Need to better handle close down gracefully but this will do for now.
fn closedown() {
    println!("Shutting Down, GoodBye!\n");
    std::process::exit(0);
}

fn main() {
    println!("Loading...");

    // Check the DB!
    let db = db::init(SOFTWARE_VERSION);

    ctrlc::set_handler(move || {
        closedown();
    })
    .expect("Error setting Ctrl-C handler");

    let sock = match UdpSocket::bind("0.0.0.0:55555") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("There was an error binding: {}", e);
            std::process::exit(-1);
        }
    };

    let mut dvec: Vec<[u8; 55]> = Vec::new();
    let mut replay_counter = 0;
    let mut d_counter = 31;
    let mut payload_counter: usize = 0;
    let mut stats_timer = SystemTime::now();

    let mut mash: HashMap<u32, Peer> = HashMap::new();
    let mut logins: HashSet<u32> = HashSet::new();

    loop {
        // Print stats at least every 1 minute and check if a peer needs removing
        match stats_timer.elapsed() {
            Ok(t) => {
                if t.as_secs() >= 60 {
                    println!("Number of logins: {}", logins.len());
                    for (t, p) in &mash {
                        println!(
                            "Peer details\n\nID: {}\nCall: {}\nTG active {:?}",
                            t, p.Callsign, p.talk_groups
                        );
                    }
                    stats_timer = SystemTime::now();
                    mash.retain(|_, p| //logins.contains(&k)
                match p.last_check.elapsed(){
                Ok(lc) => {
                    if lc.as_secs() > 15 {
                        logins.remove(&p.id);
                        false
                    } else {
                        p.talk_groups.retain(|_, t|{
                            if t.ua {
                                return match t.time_stamp.elapsed(){
                                Ok(ts) => {
                                    if ts.as_secs() > t.expire {
                                        println!("Removing TG: {}, From Peer: {}", t.id, p.id);
                                        false
                                    } else {
                                        true
                                    }
                                },
                                Err(_) => {
                                    println!("There was an error passing time for UA, removing TG!");
                                    false
                                },
                            }
                            }
                            true
                    });
                        true
                    }
                },
                Err(e) => {
                    eprintln!("Error parsing last check time: {}", e);
                    false
                }
                });
                }
            }
            Err(_) => {}
        }

        clock(&mut logins, &mut mash);
        let mut rx_buff = [0; 500];

        let (rxs, src) = match sock.recv_from(&mut rx_buff) {
            Ok(rs) => (rs),

            Err(e) => {
                eprintln!("There was an error listening: {}", e);
                std::process::exit(-1);
            }
        };

        // If we have a message play it back
        if !dvec.is_empty() && replay_counter > 1 {
            replay_counter = 0;
            echo(&sock, src, &dvec);
            println!("echo");
            dvec.clear();
        }

        payload_counter += 1;
        if !dvec.is_empty() {
            replay_counter += 1;
        }
        match &rx_buff[..4] {
            hb::DMRA => {
                println!("Todo! 1");
            }
            hb::DMRD => {
                let hbp = hb::DMRDPacket::parse(rx_buff);
                d_counter += 1;
                replay_counter = 0;
                let _packet_data = &rx_buff[..53];
                let rf_src = &rx_buff[5..8];
                let dst_tg = &rx_buff[8..11];
                let packet_seq = &rx_buff[4];

                if d_counter > 32 {
                    d_counter = 0;
                    println!(
                        "DEBUG: rf_src: {}, dest: {}, packet seq: {:x?} slot: {}, ctype: {}, stream id: {} payload count: {}",
                        hbp.src, hbp.dst, hbp.seq, hbp.sl, hbp.ct, hbp.si, payload_counter
                    );
                }
                let tx_buff: [u8; 55] = <[u8; 55]>::try_from(&rx_buff[..55]).unwrap();
                //let tx_buff = hbp.construct();

                // Repeat to peers who are members of the same talkgroup
                for (_, p) in &mut mash {
                    match p.talk_groups.get_mut(&hbp.dst) {
                        Some(tg) => {
                            if tg.sl == hbp.sl
                                && p.peer_type == tg.routeable
                                && p.ip != src
                                && p.ip
                                    != std::net::SocketAddr::new(
                                        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
                                        0,
                                    )
                            {
                                sock.send_to(&tx_buff, p.ip).unwrap();
                            } else if tg.ua {
                                // Reset the time stamp for the UA talkgroup
                                tg.time_stamp = SystemTime::now();
                            }
                        }
                        None => {
                            // If no talkgroup is found for the peer then we subscribe the peer to the talkgroup requested.
                            // If the peer does not request this talkgroup again in a 15 minute window the peer is auto-
                            // matically unsubscribed.
                            if p.ip == src {
                                p.talk_groups.insert(
                                    hbp.dst,
                                    Talkgroup::set(hbp.sl, TgActivate::Ua(hbp.dst)),
                                );
                                println!(
                                    "Added TG: {} to peer: id-{} call-{} ",
                                    &hbp.dst, &p.id, &p.Callsign
                                );
                            }
                        }
                    }
                }

                if hbp.dst == 9990 && hbp.sl == 2 {
                    dvec.push(tx_buff);
                }
            }
            hb::MSTCL => {
                println!("Todo!2");
            }
            hb::MSTNAK => {
                println!("Todo!3");
            }
            hb::MSTPONG => {
                println!("Todo!4");
            }
            hb::MSTN => {
                println!("Todo!4a");
            }
            hb::MSTP => {
                println!("Todo!5");
            }
            hb::MSTC => {
                println!("Todo!6");
            }
            hb::RPTL => {
                let mut peer = Peer::new();
                peer.pid(&<[u8; 4]>::try_from(&rx_buff[4..8]).unwrap());
                let randid = [0x0A, 0x7E, 0xD4, 0x98];
                println!("Sending Ack: {}", src);
                println!("Repeater Login Request: {:x?}", rx_buff);
                sock.send_to(&[hb::RPTACK, &rx_buff[4..8], &randid].concat(), src)
                    .unwrap();
            }
            hb::RPTPING => {
                println!("Todo!6");
            }
            hb::RPTCL => {
                println!("Todo!7");
            }
            hb::RPTACK => {
                println!("Todo!8");
            }
            hb::RPTK => {
                let mut peer = Peer::new();
                peer.pid(&<[u8; 4]>::try_from(&rx_buff[4..8]).unwrap());
                if !peer.acl() {
                    println!("Peer ID: {} is blocked", peer.id);
                    sock.send_to(&[hb::MSTNAK, &rx_buff[4..8]].concat(), src)
                        .unwrap();
                    continue;
                }
                println!("Peer: {} has logged in", peer.id);

                if logins.insert(peer.id) {
                    sock.send_to(&[hb::RPTACK, &rx_buff[4..8]].concat(), src)
                        .unwrap();
                }
            }
            hb::RPTC => {
                let mut peer = Peer::new();
                peer.pid(&<[u8; 4]>::try_from(&rx_buff[4..8]).unwrap());
                peer.ip = src;

                if !logins.contains(&peer.id) {
                    println!("Unknown peer sent info {}", peer.id);
                    continue;
                }

                peer.Callsign = match str::from_utf8(&rx_buff[8..16]) {
                    Ok(c) => c.to_owned(),
                    Err(_) => "Unknown".to_owned(),
                };
                peer.Frequency = match str::from_utf8(&rx_buff[16..38]) {
                    Ok(c) => c.to_owned(),
                    Err(_) => "Unknown".to_owned(),
                };
                println!("Callsign is: {}", peer.Callsign);
                println!("Frequency is: {}", peer.Frequency);

                mash.insert(peer.id, peer);

                sock.send_to(&[hb::RPTACK, &rx_buff[4..8]].concat(), src)
                    .unwrap();
            }
            hb::RPTP => {
                let mut peer = Peer::new();
                peer.pid(&<[u8; 4]>::try_from(&rx_buff[7..11]).unwrap());

                // Only send pong if we know about the peer.
                // Also update the peer IP address if it has changed.
                peer.ip = match mash.get_mut(&peer.id) {
                    Some(p) => {
                        p.last_check = SystemTime::now();
                        if p.ip != src {
                            p.ip = src;
                        }
                        p.ip
                    }
                    None => continue,
                };

                println!("Sending Pong");
                sock.send_to(&[hb::MSTPONG, &rx_buff[4..8]].concat(), peer.ip)
                    .unwrap();
            }
            hb::RPTA => {
                println!("Todo!10");
            }
            hb::RPTO => {
                println!("Todo!11");
            }
            hb::RPTS => {
                println!("Todo!12");
            }
            hb::RPTSBKN => {
                println!("Todo!13");
            }
            _ => {
                match std::str::from_utf8(&rx_buff[..4]) {
                    Ok(s) => println!("Unknown packet? {}", s),
                    Err(_) => eprintln!("Unknown packet header"),
                }
                payload_counter -= 1;
            }
        }
    }
}
