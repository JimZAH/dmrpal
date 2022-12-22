use dmrpal::{
    dprint, echo,
    peers::{Peer, Peertype},
    sleep, slot, streams, system,
    talkgroups::{Talkgroup, TgActivate},
};
use std::collections::hash_map::HashMap;
use std::net::{IpAddr, Ipv4Addr, UdpSocket};
use std::{env::args, io, str, time::SystemTime};

mod hb;

const USERACTIVATED_DISCONNECT_TG: u32 = 4000;

#[derive(Debug, PartialEq)]
enum Masterstate {
    Disable,
    LoginRequest,
    LoginPassword,
    Options,
    Connected,
    WaitingPong,
    Logout,
}

// Need to better handle close down gracefully but this will do for now.
fn closedown() {
    std::process::exit(0);
}

fn main() {
    let config = system::Config::load();
    println!(
        "My ID: {} | Master IP: {} | Verbose: {}",
        config.my_id, config.master_ip, config.verbose
    );
    let arg: Vec<String> = args().collect();
    let mut verbose: u8 = config.verbose;
    if arg.len() > 1 {
        match arg[1].as_ref() {
            "--verbose" | "-v" => match arg[2].parse::<u8>() {
                Ok(v) => verbose = v,
                Err(_) => dprint!(verbose;2;"Unable to set verbosity, using default"),
            },
            _ => {}
        }
    }
    dprint!(verbose;4;"Loading...");

    let mut state: Masterstate = Masterstate::Disable;

    let mut streams = streams::Streams::init();

    let mut system = system::System::init();

    let mut mash: HashMap<u32, Peer> = HashMap::new();

    let master_ip: std::net::SocketAddr = config.master_ip.parse().unwrap();

    if config.my_id != 0 {
        let mut master = Peer::new();
        master.enabled = true;
        master.callsign = "PHOENIXF".to_owned();
        master.id = config.my_id;
        master.ip = master_ip;
        master.last_check = SystemTime::now();
        master.peer_type = Peertype::All;
        master.software = "IPSC2".to_owned();
        master.talk_groups = HashMap::from([
            (23526, Talkgroup::set(1, TgActivate::Static(23526), None)),
            (2351, Talkgroup::set(1, TgActivate::Static(2351), None)),
            (235, Talkgroup::set(1, TgActivate::Static(235), None)),
            (840, Talkgroup::set(2, TgActivate::Static(840), None)),
            (841, Talkgroup::set(2, TgActivate::Static(841), None)),
            (844, Talkgroup::set(2, TgActivate::Static(844), None)),
            (123, Talkgroup::set(1, TgActivate::Static(123), None)),
            (113, Talkgroup::set(1, TgActivate::Static(113), None)),
            (80, Talkgroup::set(1, TgActivate::Static(80), None)),
            (81, Talkgroup::set(1, TgActivate::Static(81), None)),
            (82, Talkgroup::set(1, TgActivate::Static(82), None)),
            (83, Talkgroup::set(1, TgActivate::Static(83), None)),
            (84, Talkgroup::set(1, TgActivate::Static(84), None)),
            (3, Talkgroup::set(1, TgActivate::Static(3), None)),
            (2, Talkgroup::set(1, TgActivate::Static(2), None)),
            (1, Talkgroup::set(1, TgActivate::Static(1), None)),
        ]);
        master.options = "TS1_1=23526".to_owned();
        // Insert the master into mash
        mash.insert(config.my_id, master);
        state = Masterstate::LoginRequest;
    }

    ctrlc::set_handler(move || {
        closedown();
    })
    .expect("Error setting Ctrl-C handler");

    let sock = match UdpSocket::bind("0.0.0.0:55555") {
        Ok(s) => s,
        Err(e) => {
            dprint!(verbose;1;"There was an error binding: {}", e);
            std::process::exit(-1);
        }
    };

    sock.set_nonblocking(true).unwrap();

    let mut d_counter = 31;
    let mut payload_counter: usize = 0;
    let mut stats_timer = SystemTime::now();

    // This needs to be automatic but for now lets be dirty and set manually.
    let dirty_master_options: bool = true;

    let myid = hb::RPTLPacket { id: config.my_id };

    let mut rx_buff = [0; hb::RX_BUFF_MAX];

    loop {
        // Print stats at least every 1 minute and check if a peer needs removing
        match stats_timer.elapsed() {
            Ok(t) => {
                if t.as_secs() >= 60 {
                    dprint!(verbose;4;"Number of logins: {}", mash.len());
                    for (t, p) in &mash {
                        dprint!(verbose;4;
                            "Peer details\n\nID: {}\nCall: {}\nRX: {} TX: {}\nIP: {}",
                            t, p.callsign, p.rx_bytes, p.tx_bytes, p.ip
                        );

                        dprint!(verbose;4;"Total Number of streams processed: {}", streams.total);
                    }
                    stats_timer = SystemTime::now();
                    mash.retain(|_, p| match p.last_check.elapsed() {
                        Ok(lc) => {
                            if lc.as_secs() > 15 && p.id != config.my_id {
                                false
                            } else {
                                //p.talk_groups.ua_clear();
                                p.talk_groups.retain(|_, t| t.ua_clear());
                                true
                            }
                        }
                        Err(e) => {
                            dprint!(verbose;2;"Error parsing last check time: {}",e);
                            false
                        }
                    });
                }
            }
            Err(_) => {}
        }

        let (rx_byte, src) = match sock.recv_from(&mut rx_buff) {
            Ok(rs) => {
                payload_counter += 1;
                rs
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => (
                0,
                std::net::SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
            ),
            Err(e) => {
                dprint!(verbose;1;"There was an error listening: {}", e);
                std::process::exit(-1);
            }
        };

        // check the state of master connection
        if let Some(master) = mash.get_mut(&config.my_id) {
            match state {
                Masterstate::Disable => {}
                Masterstate::LoginRequest => {
                    sock.send_to(&myid.password_response(rx_buff), master.ip)
                        .unwrap();
                    dprint!(verbose;4;"sending password");
                    system.master_reconnects += 1;
                    sleep(10000);
                }
                Masterstate::LoginPassword => {
                    sock.send_to(&myid.info(), master.ip).unwrap();
                    dprint!(verbose;4;"sending info");
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
                        dprint!(verbose;2;"Error passing master last check time");
                    }
                },
                Masterstate::WaitingPong => match master.last_check.elapsed() {
                    Ok(t) => {
                        if t.as_secs() > 30 {
                            state = Masterstate::Logout;
                        }
                    }
                    Err(_) => {
                        dprint!(verbose;2;"Error passing master last check time");
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
                        config.my_id,
                        "TS1_1=23526;TS1_2=1;TS1_3=235;TS2_1=840;TS2_2=841;TS2_3=844;".to_string(),
                    );
                    dprint!(verbose;4;"Sending options to master");
                    sock.send_to(&options, master.ip).unwrap();
                }
            }
        }

        match &rx_buff[..4] {
            hb::DMRA => {
                dprint!(verbose;2;"Todo! 1");
            }
            hb::DMRD => {
                let hbp = hb::DMRDPacket::parse(rx_buff);

                // Check to see if the sending peer is enabled
                if mash.get(&hbp.rpt).is_none() && src != master_ip {
                    continue;
                }

                if streams.stream(hbp.si) {
                    dprint!(verbose;3;"Stream: {}, Timeout", hbp.si);
                    continue;
                }
                d_counter += 1;

                if d_counter > 32 {
                    d_counter = 0;
                    dprint!(verbose;10;
                        "rf_src: {}, dest: {}, packet seq: {:x?} slot: {}, ctype: {}, stream id: {} payload count: {}",
                        hbp.src, hbp.dst, hbp.seq, hbp.sl, hbp.ct, hbp.si, payload_counter
                    );
                }
                let mut tx_buff: [u8; 55] = <[u8; 55]>::try_from(&rx_buff[..55]).unwrap();

                // Repeat to peers who are members of the same talkgroup and peer type.
                for p in mash.values_mut() {
                    // Only repeat to peers which are enabled
                    if !p.enabled {
                        continue;
                    }
                    // Check we can lock slot. If the peer is simplex check if either slot is locked
                    match p.talk_groups.get_mut(&hbp.dst) {
                        Some(tg) => {
                            match hbp.sl {
                                1 => {
                                    if p.duplex == 4 {
                                        if !p.slot.lock(slot::Slots::One(hbp.dst))
                                            || !p.slot.lock(slot::Slots::Two(hbp.dst))
                                        {
                                            continue;
                                        }
                                    } else {
                                        if !p.slot.lock(slot::Slots::One(hbp.dst)) {
                                            continue;
                                        }
                                    }
                                }
                                2 => {
                                    if p.duplex == 4 {
                                        if !p.slot.lock(slot::Slots::Two(hbp.dst))
                                            || !p.slot.lock(slot::Slots::One(hbp.dst))
                                        {
                                            continue;
                                        }
                                    } else {
                                        if !p.slot.lock(slot::Slots::Two(hbp.dst)) {
                                            continue;
                                        }
                                    }
                                }
                                _ => {
                                    eprintln!("Can't lock slot, invalid slot number!");
                                    continue;
                                }
                            }
                            if tg.sl == hbp.sl
                                && p.ip != src
                                && p.ip
                                    != std::net::SocketAddr::new(
                                        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
                                        0,
                                    )
                            {
                                // If we are sending to the master we need to rewrite the source ID
                                if p.id == config.my_id {
                                    tx_buff[11..15].copy_from_slice(&p.id.to_be_bytes());
                                }
                                match sock.send_to(&tx_buff, p.ip) {
                                    Ok(s) => p.rx_bytes += s,
                                    Err(em) => {
                                        dprint!(verbose;2;"Error: {} sending to peer: {}", em, p.id)
                                    }
                                }
                                tg.la = SystemTime::now();
                            } else if tg.ua {
                                // Reset the time stamp for the UA talkgroup
                                tg.time_stamp = SystemTime::now();
                            }

                            if p.ip == src {
                                p.tx_bytes += rx_byte;
                            }
                        }
                        None => {
                            // If no talkgroup is found for the peer then we subscribe the peer to the talkgroup requested.
                            // If the peer does not request this talkgroup again in a 15 minute window the peer is auto-
                            // matically unsubscribed.
                            if p.ip == src && hbp.dst != USERACTIVATED_DISCONNECT_TG {
                                p.talk_groups.insert(
                                    hbp.dst,
                                    Talkgroup::set(
                                        hbp.sl,
                                        TgActivate::Ua(hbp.dst),
                                        Some(p.tg_expire),
                                    ),
                                );
                                dprint!(verbose;4;
                                    "Added TG: {} to peer: id-{} call-{} ",
                                    &hbp.dst, &p.id, &p.callsign
                                );
                            } else if hbp.dst == USERACTIVATED_DISCONNECT_TG {
                                // Remove all UA
                                p.talk_groups.retain(|_, t| t.ua_clear());
                            }
                        }
                    }
                    if hbp.dst == 9990 && hbp.sl == 2 && p.id == hbp.rpt && p.id != config.my_id {
                        dprint!(verbose;10;"{:X?}", &rx_buff[..55]);
                        p.echo(<[u8; 55]>::try_from(&rx_buff[..55]).unwrap(), hbp.si);
                    }
                }
            }
            hb::MSTN => {
                dprint!(verbose;2;"Todo!4a");
            }
            hb::MSTP => {
                if let Some(master) = mash.get_mut(&config.my_id) {
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
                // Just send a predefined (random string). This should be random!
                let randid = [0x0A, 0x7E, 0xD4, 0x98];
                sock.send_to(&[hb::RPTACK, &rx_buff[4..8], &randid].concat(), src)
                    .unwrap();
            }
            hb::RPTK => {
                let mut peer = Peer::new();
                peer.pid(&<[u8; 4]>::try_from(&rx_buff[4..8]).unwrap());
                if !peer.acl() {
                    dprint!(verbose;3;"Peer ID: {} is blocked", peer.id);
                    sock.send_to(&[hb::MSTNAK, &rx_buff[4..8]].concat(), src)
                        .unwrap();
                    continue;
                }
                dprint!(verbose;4;"Peer: {} has logged in", peer.id);
                peer.ip = src;
                mash.insert(peer.id, peer);
                sock.send_to(&[hb::RPTACK, &rx_buff[4..8]].concat(), src)
                    .unwrap();
            }
            hb::RPTC => {
                let mut peer = Peer::new();
                peer.pid(&<[u8; 4]>::try_from(&rx_buff[4..8]).unwrap());

                match mash.get_mut(&peer.id) {
                    Some(p) => {
                        p.enabled = true;
                        p.callsign = match str::from_utf8(&rx_buff[8..16]) {
                            Ok(c) => c.to_owned(),
                            Err(_) => "Unknown".to_owned(),
                        };
                        p.duplex = rx_buff[97] - 48;
                        p.frequency = match str::from_utf8(&rx_buff[16..38]) {
                            Ok(c) => c.to_owned(),
                            Err(_) => "Unknown".to_owned(),
                        };
                        dprint!(verbose;4;"Callsign is: {}", p.callsign);
                        dprint!(verbose;4;"Frequency is: {}", p.frequency);
                        dprint!(verbose;4;"Peer duplex type is: {}", p.duplex);

                        // To help set the correct offsets print info received in bytes
                        dprint!(verbose;10;"Peer details raw");
                        for (a, b) in rx_buff.iter().enumerate() {
                            print!("{a}:{b:X}  ");
                        }
                        dprint!(verbose;10;"\n");
                    }
                    None => {
                        dprint!(verbose;4;"Unknown peer sent info {}", peer.id);
                        continue;
                    }
                }
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
                        dprint!(verbose;3;"RPTACK RECEIVED: UNKNOWN Masterstate");
                        dprint!(verbose;3;"Master state: {:?}", state);
                        Masterstate::Disable
                    }
                }
            }
            hb::RPTO => {
                let mut peer = Peer::new();
                let peer_options = hb::RPTOPacket::parse(rx_buff);
                peer.pid(&<[u8; 4]>::try_from(&rx_buff[4..8]).unwrap());
                dprint!(verbose;4;"Peer {}; has sent options:", peer.id);
                match mash.get_mut(&peer.id) {
                    Some(p) => {
                        p.options = peer_options.options;
                        p.options();
                        sock.send_to(&[hb::RPTACK, &rx_buff[4..8]].concat(), src)
                            .unwrap();
                    }
                    None => continue,
                };
            }
            hb::RPTS => {
                dprint!(verbose;2;"Todo!12");
            }
            _ => {
                sleep(500);
                /* If a peer has an echo Queue to play then process it in quiet time. The queue is only played after 5 seconds has passed since the user recorded the message.
                Only when the queue has been played do we then drop the queue by replacing with the default.
                This isn't really an efficient way of doing this but it works for now, we can always improve later.
                 */
                for p in mash.values_mut() {
                    if !p.echo.has_items() {
                        if let Ok(t) = p.echo.la_time.elapsed() {
                            if t.as_secs() >= 5 {
                                if p.lock(p.id, 2) {
                                    continue;
                                }
                                dprint!(verbose;10;"Sending echo to peer: {}", p.id);
                                for i in &p.echo.echos {
                                    dprint!(verbose;10;"{:X?}", &i.data);
                                    sock.send_to(&i.data, p.ip).unwrap();
                                }
                                p.echo = echo::Queue::default();
                            }
                        }
                    }
                }

                streams.check();
            }
        }
        rx_buff = [0; hb::RX_BUFF_MAX];
    }
}
