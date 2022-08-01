use std::collections::{hash_map::HashMap, hash_set::HashSet};
use std::net::{IpAddr, Ipv4Addr, UdpSocket};
use std::{str, string, time::SystemTime};

mod db;
//mod hb;

const SL_OFFSET: usize = 15;
const DMRA: &[u8] = b"DMRA";
const DMRD: &[u8] = b"DMRD";
const MSTCL: &[u8] = b"MSTCL";
const MSTACK: &[u8] = b"MSTACK";
const MSTNAK: &[u8] = b"MSTNAK";
const MSTPONG: &[u8] = b"MSTPONG";
const MSTN: &[u8] = b"MSTN";
const MSTP: &[u8] = b"MSTP";
const MSTC: &[u8] = b"MSTC";
const RPTL: &[u8] = b"RPTL";
const RPTPING: &[u8] = b"RPTPING";
const RPTCL: &[u8] = b"RPTCL";
const RPTACK: &[u8] = b"RPTACK";
const RPTK: &[u8] = b"RPTK";
const RPTC: &[u8] = b"RPTC";
const RPTP: &[u8] = b"RPTP";
const RPTA: &[u8] = b"RPTA";
const RPTO: &[u8] = b"RPTO";
const RPTS: &[u8] = b"RPTS";
const RPTSBKN: &[u8] = b"RPTSBKN";

const SOFTWARE_VERSION: u64 = 1;

enum Serverstate {
    Idle,
    Inuse,
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
    talk_groups: Vec<u32>,
}

impl Serverstate {
    fn start() -> Self {
        Self::Idle
    }
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
            talk_groups: vec![235, 9, 840],
        }
    }

    // Check if the peer is allowed to sign in.
    fn acl(&self) -> bool {
        let known_peers = vec![235165, 234053702, 2340537, 2351671, 234053703, 234890901, 2352285, 2350454];
        for k in known_peers {
            if self.id.eq(&k) {
                return true;
            }
        }
        false
    }

    // Set the peer ID
    fn pid(&mut self, buff: &[u8; 4]) {
        self.id = ((buff[0] as u32) << 24)
            | ((buff[1] as u32) << 16)
            | ((buff[2] as u32) << 8)
            | (buff[3] as u32);
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
        match stats_timer.elapsed(){
            Ok(t) => {
                if t.as_secs() >= 60 {
                    println!("Number of logins: {}", logins.len());
                    stats_timer = SystemTime::now();
                    mash.retain(|&k, p| //logins.contains(&k)
                match p.last_check.elapsed(){
                Ok(lc) => {
                    if lc.as_secs() > 15 {
                        false
                    } else {
                        true
                    }
                },
                Err(e) => eprintln!("Error parsing last check time: {}", e)
                });
                }
            },
            Err(_) => {}
        }

        clock(&mut logins, &mut mash);
        let mut rx_buff = [0; 500];
        let (_, src) = match sock.recv_from(&mut rx_buff) {
            Ok(rs) => (rs),
            Err(e) => {
                eprintln!("There was an error binding: {}", e);
                std::process::exit(-1);
            }
        };

        payload_counter += 1;

        // If we have a message play it back
        if !dvec.is_empty() && replay_counter > 1 {
            replay_counter = 0;
            echo(&sock, src, &dvec);
            println!("echo");
            dvec.clear();
        }

        if !dvec.is_empty() {
            replay_counter += 1;
        }

        match &rx_buff[..4] {
            DMRA => {
                println!("Todo! 1");
            }
            DMRD => {
                d_counter += 1;
                replay_counter = 0;
                let _packet_data = &rx_buff[..53];
                let rf_src = &rx_buff[5..8];
                let dst_tg = &rx_buff[8..11];
                let packet_seq = &rx_buff[4];

                // Get ID and destination
                let rfs =
                    ((rf_src[0] as u32) << 16) | ((rf_src[1] as u32) << 8) | (rf_src[2] as u32);
                let did =
                    ((dst_tg[0] as u32) << 16) | ((dst_tg[1] as u32) << 8) | (dst_tg[2] as u32);

                let t_bits = rx_buff[SL_OFFSET];
                let mut slot = 0;

                let mut c_type = "";

                if t_bits & 0x80 == 0x80 {
                    slot = 2;
                } else {
                    slot = 1;
                }

                if t_bits & 0x40 == 0x40 {
                    c_type = "unit";
                } else if (t_bits & 0x23) == 0x23 {
                    c_type = "vcsbk";
                } else {
                    c_type = "group";
                }

                if d_counter > 32 {
                    d_counter = 0;
                    println!(
                        "DEBUG: rf_src: {}, dest: {}, packet seq: {:x?} slot: {}, ctype: {}, payload count: {}",
                        rfs, did, packet_seq, slot, c_type, payload_counter
                    );
                }
                let tx_buff: [u8; 55] = <[u8; 55]>::try_from(&rx_buff[..55]).unwrap();
                // Repeat to peers who are members of the same talkgroup
                for (_, p) in &mash {
                    if p.ip != src
                        && p.ip
                            != std::net::SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0)
                    {
                        for t in &p.talk_groups {
                            if t == &did {
                                sock.send_to(&tx_buff, p.ip).unwrap();
                            }
                        }
                    }
                }

                if did == 9 && slot == 2 {
                    dvec.push(tx_buff);
                }
            }
            MSTCL => {
                println!("Todo!2");
            }
            MSTNAK => {
                println!("Todo!3");
            }
            MSTPONG => {
                println!("Todo!4");
            }
            MSTN => {
                println!("Todo!4a");
            }
            MSTP => {
                println!("Todo!5");
            }
            MSTC => {
                println!("Todo!6");
            }
            RPTL => {
                let mut peer = Peer::new();
                peer.pid(&<[u8; 4]>::try_from(&rx_buff[4..8]).unwrap());
                let randid = [0x0A, 0x7E, 0xD4, 0x98];
                println!("Sending Ack: {}", src);
                println!("Repeater Login Request: {:x?}", rx_buff);
                sock.send_to(&[RPTACK, &rx_buff[4..8], &randid].concat(), src)
                    .unwrap();
            }
            RPTPING => {
                println!("Todo!6");
            }
            RPTCL => {
                println!("Todo!7");
            }
            RPTACK => {
                println!("Todo!8");
            }
            RPTK => {
                let mut peer = Peer::new();
                peer.pid(&<[u8; 4]>::try_from(&rx_buff[4..8]).unwrap());
                if !peer.acl() {
                    println!("Peer ID: {} is not known to us!", peer.id);
                    sock.send_to(&[MSTNAK, &rx_buff[4..8]].concat(), src)
                        .unwrap();
                    continue;
                }
                println!("Peer: {} has logged in", peer.id);

                if logins.insert(peer.id) {
                    sock.send_to(&[RPTACK, &rx_buff[4..8]].concat(), src)
                        .unwrap();
                }
            }
            RPTC => {
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

                sock.send_to(&[RPTACK, &rx_buff[4..8]].concat(), src)
                    .unwrap();
            }
            RPTP => {
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
                sock.send_to(&[MSTPONG, &rx_buff[4..8]].concat(), peer.ip)
                    .unwrap();
            }
            RPTA => {
                println!("Todo!10");
            }
            RPTO => {
                println!("Todo!11");
            }
            RPTS => {
                println!("Todo!12");
            }
            RPTSBKN => {
                println!("Todo!13");
            }
            _ => {
                // This needs to be adjusted soon as it will panic if the first 4 bytes are not UTF-8
                let u_packet = std::str::from_utf8(&rx_buff[..4]).unwrap();
                println!("Unknown packet? {}", u_packet);
                payload_counter -= 1;
            }
        }
    }
}
