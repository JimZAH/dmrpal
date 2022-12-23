use hmac_sha256::Hash;

pub const DMRA: &[u8] = b"DMRA";
pub const DMRD: &[u8] = b"DMRD";
pub const MSTNAK: &[u8] = b"MSTNAK";
pub const MSTPONG: &[u8] = b"MSTPONG";
pub const MSTN: &[u8] = b"MSTN";
pub const MSTP: &[u8] = b"MSTP";
pub const MSTC: &[u8] = b"MSTC";
pub const RPTL: &[u8] = b"RPTL";
pub const RPTPING: &[u8] = b"RPTPING";
pub const RPTACK: &[u8] = b"RPTACK";
pub const RPTK: &[u8] = b"RPTK";
pub const RPTC: &[u8] = b"RPTC";
pub const RPTP: &[u8] = b"RPTP";
pub const RPTA: &[u8] = b"RPTA";
pub const RPTO: &[u8] = b"RPTO";
pub const RPTS: &[u8] = b"RPTS";

pub const RX_BUFF_MAX: usize = 512;

// DMRD paclet structure
pub struct DMRDPacket {
    pub seq: u8,
    pub src: u32,
    pub dst: u32,
    pub rpt: u32,
    pub sl: u8,
    pub ct: u8,
    pub ft: u8,
    pub dt: u8,
    pub si: u32,
    pub dd: [u8; 35],
}

pub struct RPTCPacket {
    callsign: [u8; 8],
    rptrid: [u8; 4],
    rx_freq: [u8; 9],
    tx_freq: [u8; 9],
    tx_pwr: [u8; 2],
    color_code: [u8; 2],
    latitude: [u8; 8],
    longitude: [u8; 9],
    height: [u8; 3],
    location: [u8; 20],
    description: [u8; 20],
    url: [u8; 124],
    software_id: [u8; 40],
    package_id: [u8; 40],
}

pub struct RPTLPacket {
    pub id: u32,
}

pub struct RPTOPacket {
    pub id: u32,
    pub options: String,
}

impl DMRDPacket {
    // TODO: Data type 4 bits
    pub fn construct(&self) -> [u8; 55] {
        let mut cbuf = [0; 55];

        cbuf[0] = 'D' as u8;
        cbuf[1] = 'M' as u8;
        cbuf[2] = 'R' as u8;
        cbuf[3] = 'D' as u8;

        cbuf[4] = self.seq;

        /*
        Rust doesn't have a 24bit type in the standard library
        There is a crate that adds this custom type, but I don't
        see a need to add a crate when we can just shift the
        numbers manually.
        */
        cbuf[5] = (self.src >> 16) as u8;
        cbuf[6] = (self.src >> 8) as u8;
        cbuf[7] = (self.src >> 0) as u8;
        cbuf[8] = (self.dst >> 16) as u8;
        cbuf[9] = (self.dst >> 8) as u8;
        cbuf[10] = (self.dst >> 0) as u8;

        cbuf[11..15].copy_from_slice(&self.rpt.to_be_bytes());

        if self.sl == 2 {
            cbuf[15] = 1;
        }

        if self.ct == 1 {
            cbuf[15] |= 1 << 1;
        }

        if self.ft == 1 {
            cbuf[15] |= 1 << 2;
        } else if self.ft == 2 {
            cbuf[15] |= 1 << 3;
        } else if self.ft == 3 {
            cbuf[15] |= 3 << 2;
        }

        cbuf[16..20].copy_from_slice(&self.si.to_be_bytes());
        cbuf[20..55].copy_from_slice(&self.dd);
        cbuf
    }

    // Parse DMRD packet
    // TODO: Data type 4 bits
    pub fn parse(buf: [u8; RX_BUFF_MAX]) -> Self {
        let mut c_type = 0;
        let mut f_type = 0;
        let mut slot = 1;

        if buf[15] & 0x80 == 0x80 {
            slot = 2;
        }

        if buf[15] & 0x40 == 0x40 {
            c_type = 1;
        } else if (buf[15] & 0x23) == 0x23 {
            f_type = 1;
        } else {
            c_type = 0;
        }

        let mut dmrd = [0; 35];
        dmrd.clone_from_slice(&buf[20..55]);

        Self {
            seq: buf[4],
            src: ((buf[5] as u32) << 16) | ((buf[6] as u32) << 8) | (buf[7] as u32),
            dst: ((buf[8] as u32) << 16) | ((buf[9] as u32) << 8) | (buf[10] as u32),
            rpt: ((buf[11] as u32) << 24)
                | ((buf[12] as u32) << 16)
                | ((buf[13] as u32) << 8)
                | (buf[14] as u32),
            sl: slot,
            ct: c_type,
            ft: f_type,
            dt: 0,
            si: ((buf[16] as u32) << 24)
                | ((buf[17] as u32) << 16)
                | ((buf[18] as u32) << 8)
                | (buf[19] as u32),
            dd: dmrd,
        }
    }
}

impl RPTLPacket {
    pub fn request_login(&self) -> [u8; 8] {
        let mut b = [0; 8];
        b[0] = b'R';
        b[1] = b'P';
        b[2] = b'T';
        b[3] = b'L';

        b[4..].copy_from_slice(&self.id.to_be_bytes());
        b
    }

    pub fn password_response(&self, buf: [u8; RX_BUFF_MAX]) -> [u8; 40] {
        let password = b"PASSWORD";
        let mut bf = [0; 40];
        let mut pbuf = [0; 12];
        bf[0] = b'R';
        bf[1] = b'P';
        bf[2] = b'T';
        bf[3] = b'K';

        let ran = &buf[6..10];

        pbuf[0..4].copy_from_slice(ran);
        pbuf[4..12].copy_from_slice(password);

        let result = Hash::hash(&pbuf);

        bf[4..8].copy_from_slice(&self.id.to_be_bytes());
        bf[8..40].copy_from_slice(&result);

        bf
    }

    pub fn info(&self) -> [u8; 302] {
        // This is for testing only and we must write a way to extract config, this is for POC
        let mut b = [0x20; 302];
        let rx_f = b"434525";
        b[0] = b'R';
        b[1] = b'P';
        b[2] = b'T';
        b[3] = b'C';
        b[4..8].copy_from_slice(&self.id.to_be_bytes());
        b[8] = b'M';
        b[9] = b'X';
        b[10] = b'0';
        b[11] = b'W';
        b[12] = b'V';
        b[13] = b'V';
        //b[16..20].copy_from_slice(&self.id.to_be_bytes());
        b[16..rx_f.len() + 16].copy_from_slice(rx_f);
        b[38..40].copy_from_slice(b"49");
        b[40..42].copy_from_slice(b"49");
        b[42..51].copy_from_slice(b"+50.42432");
        b[51..60].copy_from_slice(b"+007.3412");
        b[60..63].copy_from_slice(b"103");
        b[63..84].copy_from_slice(b"NorwichNorwichNorwich");
        b[82..103].copy_from_slice(b"Testing system 323455");

        //97

        b[228..239].copy_from_slice(b"DMRPaL:0.1B");
        b[269..275].copy_from_slice(b"DMRPaL");
        b
    }

    pub fn info_parse(&self, buf: [u8; RX_BUFF_MAX]) {
        println!("Peer duplex type: {}", buf[97])
    }
    pub fn ping(&self) -> [u8; 8] {
        let mut b = [0; 8];
        b[0] = b'R';
        b[1] = b'P';
        b[2] = b'T';
        b[3] = b'P';
        b[4..].copy_from_slice(&self.id.to_be_bytes());
        b
    }
}

impl RPTOPacket {
    pub fn construct(id: u32, options: String) -> [u8; RX_BUFF_MAX] {
        let mut b = [0; RX_BUFF_MAX];
        let options_size = options.len();
        b[0] = b'R';
        b[1] = b'P';
        b[2] = b'T';
        b[3] = b'O';
        b[4..8].copy_from_slice(&id.to_be_bytes());
        b[8..options_size + 8].copy_from_slice(options.as_bytes());
        b
    }

    pub fn parse(buf: [u8; RX_BUFF_MAX]) -> Self {
        Self {
            id: ((buf[5] as u32) << 16) | ((buf[6] as u32) << 8) | (buf[7] as u32),
            options: String::from_utf8_lossy(&buf[8..]).to_string(),
        }
    }
}
