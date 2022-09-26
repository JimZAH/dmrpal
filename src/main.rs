use dmrpal::{debug, echo, master, sleep, slot, SystemState, Systemstate};
use std::collections::{hash_map::HashMap, hash_set::HashSet};
use std::net::{IpAddr, Ipv4Addr, UdpSocket};
use std::{io, str, string, time::SystemTime};

mod db;
mod hb;

const SOFTWARE_VERSION: u64 = 1;
const USERACTIVATED_DISCONNECT_TG: u32 = 4000;
const MY_ID: u32 = 235045402;
const REMOTE_PEER: &str = "78, 129, 135, 43";

#[derive(Debug, PartialEq)]
enum Peertype {
    Local,
    Friend,
    All,
}

#[derive(Debug, PartialEq)]
enum Masterstate {
    Disable,
    Disconnected,
    LoginRequest,
    LoginPassword,
    Options,
    Connected,
    WaitingPong,
    Logout,
}

enum TgActivate {
    Static(u32),
    Ua(u32),
}

struct Peer {
    id: u32,
    callsign: String,
    duplex: u8,
    frequency: String,
    software: String,
    latitude: f32,
    last_check: SystemTime,
    longitude: f32,
    power: u16,
    height: u16,
    ip: std::net::SocketAddr,
    talk_groups: HashMap<u32, Talkgroup>,
    options: String,
    peer_type: Peertype,
    slot: slot::Slot,
}

#[derive(Debug)]
struct Talkgroup {
    expire: u64,
    id: u32,
    la: SystemTime,
    routeable: Peertype,
    sl: u8,
    ua: bool,
    time_stamp: SystemTime,
}

impl Peer {
    fn new() -> Self {
        Self {
            id: 0,
            callsign: string::String::default(),
            duplex: 0,
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
                (31337, Talkgroup::set(2, TgActivate::Static(31337))),
                (23526, Talkgroup::set(1, TgActivate::Ua(23526))),
                (2351, Talkgroup::set(1, TgActivate::Static(2351))),
                (235, Talkgroup::set(1, TgActivate::Static(235))),
                (844, Talkgroup::set(2, TgActivate::Static(844))),
                (840, Talkgroup::set(2, TgActivate::Static(840))),
                (123, Talkgroup::set(1, TgActivate::Static(123))),
                (113, Talkgroup::set(1, TgActivate::Static(113))),
                (80, Talkgroup::set(1, TgActivate::Ua(80))),
                (81, Talkgroup::set(1, TgActivate::Ua(81))),
                (82, Talkgroup::set(1, TgActivate::Ua(82))),
                (83, Talkgroup::set(1, TgActivate::Ua(83))),
                (84, Talkgroup::set(1, TgActivate::Ua(84))),
                (3, Talkgroup::set(1, TgActivate::Static(3))),
                (2, Talkgroup::set(1, TgActivate::Static(2))),
                (1, Talkgroup::set(1, TgActivate::Static(1))),
            ]),
            options: string::String::default(),
            peer_type: Peertype::Local,
            slot: slot::Slot::init(),
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

    fn connect_master(&mut self) -> Masterstate {
        let myid = hb::RPTLPacket { id: MY_ID };
        let pip = std::net::SocketAddr::from(std::net::SocketAddrV4::new(
            std::net::Ipv4Addr::new(78, 129, 135, 43),
            55555,
        ));
        let mut rx_buff = [0; hb::RX_BUFF_MAX];
        let mut state = Masterstate::LoginRequest;
        let sock = match UdpSocket::bind("0.0.0.0:55555") {
            Ok(s) => s,
            Err(e) => {
                eprintln!("There was an error binding: {}", e);
                std::process::exit(-1);
            }
        };
        loop {
            sock.send_to(&myid.request_login(), pip).unwrap();

            let (_, src) = match sock.recv_from(&mut rx_buff) {
                Ok(rs) => (rs),

                Err(e) => {
                    eprintln!("There was an error listening: {}", e);
                    std::process::exit(-1);
                }
            };

            match &rx_buff[..6] {
                hb::RPTACK => match &state {
                    Masterstate::Disable => {}
                    Masterstate::Disconnected => {}
                    Masterstate::LoginRequest => {
                        sock.send_to(&myid.password_response(rx_buff), pip).unwrap();
                        println!("sending password");
                        state = Masterstate::LoginPassword;
                        sleep(80000);
                    }
                    Masterstate::LoginPassword => {
                        sock.send_to(&myid.info(), pip).unwrap();
                        println!("sending info");
                        state = Masterstate::Connected;
                        sleep(80000);
                    }
                    Masterstate::Connected => {
                        sock.send_to(&myid.ping(), pip).unwrap();
                        println!("connected");
                        self.ip = src;
                        break;
                    }
                    Masterstate::Options => {}
                    Masterstate::Logout => {}
                    _ => {}
                },
                hb::RPTNAK => {
                    println!("MASTER Connect: Received NAK");
                    state = Masterstate::Disconnected;
                    break;
                }
                _ => println!("MASTER Connect: Packet not handled!\n{:X?}", rx_buff),
            }
        }
        state
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
            la: SystemTime::now(),
            routeable: Peertype::Local,
            sl: 1,
            ua: false,
            time_stamp: SystemTime::now(),
        }
    }

    // Remove a talkgroup from a peer
    fn ua_clear(&mut self) -> bool {
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
    fn set(sl: u8, tg: TgActivate) -> Self {
        let (ua, talk_group, exp) = match tg {
            TgActivate::Static(u) => (false, u, 0),
            TgActivate::Ua(u) => (true, u, 900),
        };

        Self {
            expire: exp,
            id: talk_group,
            la: SystemTime::now(),
            routeable: Peertype::Local,
            sl: sl,
            ua: ua,
            time_stamp: SystemTime::now(),
        }
    }
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
    let _db = db::init(SOFTWARE_VERSION);

    // Queue for Echo frames
    let mut echo_queue = echo::Queue::default();

    let mut state = Masterstate::Disconnected;

    let mut states: HashMap<u32, master::State> = HashMap::new();

    // For now (lots of these for nows) we manually create the master peer.
    let mut master = Peer::new();
    master.callsign = "PHOENIXF".to_owned();
    master.id = MY_ID;
    master.ip = std::net::SocketAddr::from(std::net::SocketAddrV4::new(
        std::net::Ipv4Addr::new(78, 129, 135, 43),
        55555,
    ));
    master.last_check = SystemTime::now();
    master.peer_type = Peertype::All;
    master.software = "IPSC2".to_owned();
    master.talk_groups = HashMap::from([
        (23526, Talkgroup::set(1, TgActivate::Static(23526))),
        (2351, Talkgroup::set(1, TgActivate::Static(2351))),
        (235, Talkgroup::set(1, TgActivate::Static(235))),
        (840, Talkgroup::set(2, TgActivate::Static(840))),
        (841, Talkgroup::set(2, TgActivate::Static(841))),
        (844, Talkgroup::set(2, TgActivate::Static(844))),
        (123, Talkgroup::set(1, TgActivate::Static(123))),
        (113, Talkgroup::set(1, TgActivate::Static(113))),
        (80, Talkgroup::set(1, TgActivate::Static(80))),
        (81, Talkgroup::set(1, TgActivate::Static(81))),
        (82, Talkgroup::set(1, TgActivate::Static(82))),
        (83, Talkgroup::set(1, TgActivate::Static(83))),
        (84, Talkgroup::set(1, TgActivate::Static(84))),
        (3, Talkgroup::set(1, TgActivate::Static(3))),
        (2, Talkgroup::set(1, TgActivate::Static(2))),
        (1, Talkgroup::set(1, TgActivate::Static(1))),
    ]);
    master.options = "TS1_1=23526".to_owned();

    if !REMOTE_PEER.is_empty() {
        // This is just a horrible POC to see if we could login as a peer. Yes we can so now the real work begins.
        state = Masterstate::LoginRequest;
    } else {
        state = Masterstate::Disable;
    }

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

    sock.set_nonblocking(true).unwrap();

    let mut dvec: Vec<[u8; 55]> = Vec::new();
    let mut replay_counter = 0;
    let mut d_counter = 31;
    let mut payload_counter: usize = 0;
    let mut stats_timer = SystemTime::now();

    let mut mash: HashMap<u32, Peer> = HashMap::new();
    let mut logins: HashSet<u32> = HashSet::new();

    // This needs to be automatic but for now lets be dirty and set manually.
    let dirty_master_options: bool = true;

    // Insert the master into mash
    mash.insert(MY_ID, master);

    let myid = hb::RPTLPacket { id: MY_ID };
    let pip = std::net::SocketAddr::from(std::net::SocketAddrV4::new(
        std::net::Ipv4Addr::new(78, 129, 135, 43),
        55555,
    ));

    let mut rx_buff = [0; hb::RX_BUFF_MAX];

    loop {
        // Print stats at least every 1 minute and check if a peer needs removing
        match stats_timer.elapsed() {
            Ok(t) => {
                if t.as_secs() >= 60 {
                    println!("Number of logins: {}", logins.len());
                    for (t, p) in &mash {
                        println!(
                            "Peer details\n\nID: {}\nCall: {}\nTG active {:?}\nOptions: {}\nIP: {}",
                            t, p.callsign, p.talk_groups, p.options, p.ip
                        );
                    }
                    stats_timer = SystemTime::now();
                    mash.retain(|_, p| //logins.contains(&k)
                match p.last_check.elapsed(){
                Ok(lc) => {
                    if lc.as_secs() > 15 && p.id != MY_ID{
                        logins.remove(&p.id);
                        false
                    } else {
                        //p.talk_groups.ua_clear();
                        p.talk_groups.retain(|_, t|{
                            t.ua_clear()
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

        let (_, src) = match sock.recv_from(&mut rx_buff) {
            Ok(rs) => {
                payload_counter += 1;
                rs
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => (
                0,
                std::net::SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
            ),
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

        if !dvec.is_empty() {
            replay_counter += 1;
        }

        // check the state of master connection

        if let Some(master) = mash.get_mut(&MY_ID) {
            match state {
                Masterstate::Disable => {}
                Masterstate::LoginRequest => {
                    sock.send_to(&myid.password_response(rx_buff), pip).unwrap();
                    println!("sending password");
                    sleep(10000);
                }
                Masterstate::LoginPassword => {
                    sock.send_to(&myid.info(), pip).unwrap();
                    println!("sending info");
                    sleep(95000);
                }
                Masterstate::Connected => match master.last_check.elapsed() {
                    Ok(t) => {
                        if t.as_secs() > 15 {
                            sock.send_to(
                                &[hb::RPTPING, &master.id.to_be_bytes()].concat(),
                                master.ip,
                            )
                            .unwrap();
                            state = Masterstate::WaitingPong;
                        }
                    }
                    Err(_) => {
                        eprintln!("Error passing master last check time");
                    }
                },
                Masterstate::WaitingPong => match master.last_check.elapsed() {
                    Ok(t) => {
                        if t.as_secs() > 30 {
                            state = Masterstate::Logout;
                        }
                    }
                    Err(_) => {
                        eprintln!("Error passing master last check time");
                    }
                },
                Masterstate::Logout => {
                    // The master logged us out so lets try logging in again after 5 minutes.
                    // TODO manage peer logins better.

                    if let Ok(t) = master.last_check.elapsed() {
                        if t.as_secs() > 300 {
                            state = Masterstate::LoginRequest;
                        }
                    }
                }
                Masterstate::Options => {
                    let options = hb::RPTOPacket::construct(
                        MY_ID,
                        "TS1_1=23526;TS1_2=1;TS1_3=235;TS2_1=840;TS2_2=841;TS2_3=844;".to_string(),
                    );
                    println!("Sending options to master");
                    sock.send_to(&options, pip).unwrap();
                }
                _ => {
                    println!("Wrong master state");
                }
            }
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

                if d_counter > 32 {
                    d_counter = 0;
                    println!(
                        "DEBUG: rf_src: {}, dest: {}, packet seq: {:x?} slot: {}, ctype: {}, stream id: {} payload count: {}",
                        hbp.src, hbp.dst, hbp.seq, hbp.sl, hbp.ct, hbp.si, payload_counter
                    );
                }
                let mut tx_buff: [u8; 55] = <[u8; 55]>::try_from(&rx_buff[..55]).unwrap();
                //let tx_buff = hbp.construct();

                // Repeat to peers who are members of the same talkgroup and peer type.
                for p in mash.values_mut() {
                    match p.talk_groups.get_mut(&hbp.dst) {
                        Some(tg) => {
                            if tg.sl == hbp.sl
                                && p.ip != src
                                && p.ip
                                    != std::net::SocketAddr::new(
                                        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
                                        0,
                                    )
                            {
                                // Check we can lock slot
                                match hbp.sl {
                                    1 => {
                                        if !p.slot.lock(slot::Slots::One(hbp.dst)) {
                                            println!("Peer {} slot 1 is already locked", p.id);
                                            continue;
                                        }
                                    }
                                    2 => {
                                        if !p.slot.lock(slot::Slots::Two(hbp.dst)) {
                                            println!("Peer {} slot 2 is already locked", p.id);
                                            continue;
                                        }
                                    }
                                    _ => {
                                        eprintln!("Can't lock slot, invalid slot number!");
                                        continue;
                                    }
                                }

                                // If we are sending to the master we need to rewrite the source ID
                                if p.id == MY_ID {
                                    tx_buff[11..15].copy_from_slice(&p.id.to_be_bytes());
                                }
                                match sock.send_to(&tx_buff, p.ip) {
                                    Ok(_) => {}
                                    Err(em) => eprintln!("Error: {} sending to peer: {}", em, p.id),
                                }
                                tg.la = SystemTime::now();
                            } else if tg.ua {
                                // Reset the time stamp for the UA talkgroup
                                tg.time_stamp = SystemTime::now();
                            }
                        }
                        None => {
                            // If no talkgroup is found for the peer then we subscribe the peer to the talkgroup requested.
                            // If the peer does not request this talkgroup again in a 15 minute window the peer is auto-
                            // matically unsubscribed.
                            if p.ip == src && hbp.dst != USERACTIVATED_DISCONNECT_TG {
                                p.talk_groups.insert(
                                    hbp.dst,
                                    Talkgroup::set(hbp.sl, TgActivate::Ua(hbp.dst)),
                                );
                                println!(
                                    "Added TG: {} to peer: id-{} call-{} ",
                                    &hbp.dst, &p.id, &p.callsign
                                );
                            } else if hbp.dst == USERACTIVATED_DISCONNECT_TG {
                                // Remove all UA
                                p.talk_groups.retain(|_, t| t.ua_clear());
                            }
                        }
                    }
                }

                if hbp.dst == 9990 && hbp.sl == 2 {
                    let f = echo::Frame::create(tx_buff, src, hbp.si);
                    f.commit(&mut echo_queue);
                }
            }
            hb::MSTN => {
                println!("Todo!4a");
            }
            hb::MSTP => {
                if let Some(master) = mash.get_mut(&MY_ID) {
                    master.last_check = SystemTime::now();
                    state = Masterstate::Connected;
                }
            }
            hb::MSTC => {
                // We've received a disconnect request from the master.
                if state == Masterstate::Connected {
                    state = Masterstate::Logout;
                }
            }
            hb::RPTL => {
                let mut peer = Peer::new();
                peer.pid(&<[u8; 4]>::try_from(&rx_buff[4..8]).unwrap());
                // Just send a predefined (random string). This needs to be random!
                let randid = [0x0A, 0x7E, 0xD4, 0x98];
                sock.send_to(&[hb::RPTACK, &rx_buff[4..8], &randid].concat(), src)
                    .unwrap();
            }
            hb::RPTCL => {
                println!("Todo!7");
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

                peer.callsign = match str::from_utf8(&rx_buff[8..16]) {
                    Ok(c) => c.to_owned(),
                    Err(_) => "Unknown".to_owned(),
                };
                peer.frequency = match str::from_utf8(&rx_buff[16..38]) {
                    Ok(c) => c.to_owned(),
                    Err(_) => "Unknown".to_owned(),
                };
                println!("Callsign is: {}", peer.callsign);
                println!("Frequency is: {}", peer.frequency);

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

                sock.send_to(&[hb::MSTPONG, &rx_buff[4..8]].concat(), peer.ip)
                    .unwrap();
            }
            hb::RPTA => {
                state = match state {
                    Masterstate::LoginRequest => Masterstate::LoginPassword,
                    Masterstate::LoginPassword => {
                        if !dirty_master_options {
                            Masterstate::Connected
                        } else {
                            Masterstate::Options
                        }
                    }
                    Masterstate::Logout => Masterstate::Logout,
                    Masterstate::Connected | Masterstate::Options => Masterstate::Connected,
                    _ => {
                        debug("RPTACK RECEIVED: UNKNOWN Masterstate");
                        println!("Master state: {:?}", state);
                        Masterstate::Disable
                    }
                }
            }
            hb::RPTO => {
                let mut peer = Peer::new();
                let peer_options = hb::RPTOPacket::parse(rx_buff);
                peer.pid(&<[u8; 4]>::try_from(&rx_buff[4..8]).unwrap());
                println!("Peer {}; has sent options:", peer.id);
                match mash.get_mut(&peer.id) {
                    Some(p) => {
                        p.options = peer_options.options;
                        sock.send_to(&[hb::RPTACK, &rx_buff[4..8]].concat(), src)
                            .unwrap();
                    }
                    None => continue,
                };
            }
            hb::RPTS => {
                println!("Todo!12");
            }
            _ => {
                sleep(200);
            }
        }
        rx_buff = [0; hb::RX_BUFF_MAX];
    }
}
