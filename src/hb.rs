pub const DMRA: &[u8] = b"DMRA";
pub const DMRD: &[u8] = b"DMRD";
pub const MSTCL: &[u8] = b"MSTCL";
pub const MSTNAK: &[u8] = b"MSTNAK";
pub const MSTPONG: &[u8] = b"MSTPONG";
pub const MSTN: &[u8] = b"MSTN";
pub const MSTP: &[u8] = b"MSTP";
pub const MSTC: &[u8] = b"MSTC";
pub const RPTL: &[u8] = b"RPTL";
pub const RPTPING: &[u8] = b"RPTPING";
pub const RPTCL: &[u8] = b"RPTCL";
pub const RPTACK: &[u8] = b"RPTACK";
pub const RPTK: &[u8] = b"RPTK";
pub const RPTC: &[u8] = b"RPTC";
pub const RPTP: &[u8] = b"RPTP";
pub const RPTA: &[u8] = b"RPTA";
pub const RPTO: &[u8] = b"RPTO";
pub const RPTS: &[u8] = b"RPTS";
pub const RPTSBKN: &[u8] = b"RPTSBKN";

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
            cbuf[15] = 0x80;
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
    pub fn parse(buf: [u8; 500]) -> Self {
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
