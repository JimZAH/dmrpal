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
    pub vs: u8,
    pub si: u32,
    pub dd: [u8; 33],
}

impl DMRDPacket {
    pub fn construct(&self) -> [u8; 55] {
        let mut cbuf = [0; 55];

        cbuf[0] = 'D' as u8;
        cbuf[1] = 'M' as u8;
        cbuf[2] = 'R' as u8;
        cbuf[3] = 'D' as u8;

        cbuf[4] = self.seq;
        
        cbuf[5..8].copy_from_slice(&self.src.to_be_bytes());
        cbuf[8..11].copy_from_slice(&self.dst.to_be_bytes());
        cbuf[11..15].copy_from_slice(&self.rpt.to_be_bytes());

        cbuf
    }

    // Parse DMRD packet
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

        let mut dmrd = [0; 33];
        dmrd.clone_from_slice(&buf[20..53]);

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
            vs: 0,
            si: ((buf[16] as u32) << 24)
                | ((buf[17] as u32) << 16)
                | ((buf[18] as u32) << 8)
                | (buf[19] as u32),
            dd: dmrd,
        }
    }
}
