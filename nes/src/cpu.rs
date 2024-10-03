use super::*;

#[derive(Debug, Clone)]
pub struct Cpu {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub pc: u16,
    pub s: u8,
    pub p: u8,
}

impl Cpu {
    pub fn new(start: Option<u16>, fffc: u8, fffd: u8) -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            pc: start.unwrap_or(((fffd as u16) << 8) | (fffc as u16)),
            s: 0xfd,
            p: 0x24,
        }
    }
}

macro_rules! addr_mode {
    ($call: tt $self: tt $name: ident $($rest: tt)*) => {{
        let addr = $self.$name();
        $self.$call(addr, $($rest)*)
    }};
}

macro_rules! set_val_nz {
    ($self: tt $($dest: expr,)+, $val: expr) => {{
        let val = $val;
        $($dest = val;)+
        $self.set_n(val);
        $self.set_z(val);
    }};
    ($self: tt $dest: expr, $op: tt $val: expr) => {{
        $dest $op $val;
        $self.set_n($dest);
        $self.set_z($dest);
    }};
}

macro_rules! dcp {
    ($self: tt $name: ident) => {{
        let addr = $self.$name();
        let m = $self.load(addr) - 1;
        $self.store(addr, m);
        $self.elapse_cycles(1);
        $self.set_n($self.cpu.a - m);
        $self.set_z($self.cpu.a - m);
        $self.cpu.p &= 0xfe;
        $self.cpu.p |= (m <= $self.cpu.a) as u8;
    }};
}

macro_rules! isc {
    ($self: tt $name: ident) => {{
        let addr = $self.$name();
        let m = $self.load(addr) + 1;
        $self.store(addr, m);
        $self.elapse_cycles(1);

        let res = $self.cpu.a as i8 as i16 - m as i8 as i16 - (1 - ($self.cpu.p & 1)) as i16;
        $self.cpu.p &= 0xbe;
        $self.cpu.p |= ((res as i8) < 0) as u8;
        $self.cpu.p |= ((res > 127 || res < -128) as u8) << 6;
        $self.cpu.a = res as u8;
        $self.set_n($self.cpu.a);
        $self.set_z($self.cpu.a);
    }};
}

macro_rules! rla {
    ($self: tt $name: ident) => {{
        let addr = $self.$name();
        let m = $self.load(addr);
        let v = (m << 1) | ($self.cpu.p & 1);
        $self.cpu.p &= 0xfe;
        $self.cpu.p |= m >> 7;
        $self.store(addr, v);
        $self.elapse_cycles(1);

        set_val_nz!($self $self.cpu.a, &= v);
    }};
}

macro_rules! rra {
    ($self: tt $name: ident) => {{
        let addr = $self.$name();
        let m = $self.load(addr);
        let v = ($self.cpu.p << 7) | (m >> 1);
        $self.store(addr, v);
        $self.elapse_cycles(1);

        let (a, c1) = $self.cpu.a.overflowing_add(v);
        let (res, c2) = a.overflowing_add(m & 1);

        $self.cpu.p &= 0xbe;
        $self.cpu.p |= (c1 | c2) as u8;
        $self.cpu.p |= ((!($self.cpu.a ^ v) & ($self.cpu.a ^ res) & 0x80) >> 1) as u8;
        set_val_nz!($self $self.cpu.a, = res);
    }};
}

macro_rules! slo {
    ($self: tt $name: ident) => {{
        let addr = $self.$name();
        let m = $self.load(addr);
        $self.store(addr, m << 1);
        $self.elapse_cycles(1);

        $self.cpu.a |= m << 1;
        $self.set_n($self.cpu.a);
        $self.set_z($self.cpu.a);

        $self.cpu.p &= 0xfe;
        $self.cpu.p |= (m >> 7) | (($self.cpu.a == 0) as u8);
    }};
}

macro_rules! sre {
    ($self: tt $name: ident) => {{
        let addr = $self.$name();
        let m = $self.load(addr);
        let v = m >> 1;
        $self.store(addr, v);
        $self.elapse_cycles(1);

        set_val_nz!($self $self.cpu.a, ^= v);
        $self.cpu.p |= m & 1;
    }};
}

impl Nes<'_> {
    // aka (zp, x)
    fn addr_of_indx_indr(&mut self) -> u16 {
        let ind = self.fetch_pc() + self.cpu.x;
        self.elapse_cycles(1);
        self.load(ind as u16) as u16 | (self.load((ind + 1) as u16) as u16) << 8
    }

    fn addr_of_zp(&mut self) -> u16 {
        self.fetch_pc() as u16
    }

    fn addr_of_abs(&mut self) -> u16 {
        self.fetch_u16()
    }

    fn addr_of_zp_x(&mut self) -> u16 {
        let off = self.fetch_pc();
        self.elapse_cycles(1);
        (off + self.cpu.x) as u16
    }

    fn addr_of_zp_y(&mut self) -> u16 {
        let off = self.fetch_pc();
        self.elapse_cycles(1);
        (off + self.cpu.y) as u16
    }

    // aka (zp), y
    fn addr_of_indr_indx(&mut self) -> u16 {
        let ind = self.fetch_pc();
        let a = self.load(ind as u16) as u16 | (self.load((ind + 1) as u16) as u16) << 8;
        self.elapse_cycles(((a as u8).overflowing_add(self.cpu.y).1) as usize);
        a + self.cpu.y as u16
    }

    fn addr_of_abs_x(&mut self) -> u16 {
        let off = self.fetch_u16();
        self.elapse_cycles(((off as u8).overflowing_add(self.cpu.x).1) as usize);
        off + self.cpu.x as u16
    }

    fn addr_of_abs_y(&mut self) -> u16 {
        let off = self.fetch_u16();
        self.elapse_cycles(((off as u8).overflowing_add(self.cpu.y).1) as usize);
        off + self.cpu.y as u16
    }

    // aka (zp), y
    fn addr_of_indr_indx_store(&mut self) -> u16 {
        let ind = self.fetch_pc();
        let a = self.load(ind as u16) as u16 | (self.load((ind + 1) as u16) as u16) << 8;
        self.elapse_cycles(1);
        a + self.cpu.y as u16
    }

    fn addr_of_abs_x_store(&mut self) -> u16 {
        let off = self.fetch_u16();
        self.elapse_cycles(1);
        off + self.cpu.x as u16
    }

    fn addr_of_abs_y_store(&mut self) -> u16 {
        let off = self.fetch_u16();
        self.elapse_cycles(1);
        off + self.cpu.y as u16
    }

    fn set_n(&mut self, v: u8) {
        self.cpu.p &= 0x7f;
        self.cpu.p |= v & 0x80;
    }

    fn set_z(&mut self, v: u8) {
        self.cpu.p &= 0xfd;
        self.cpu.p |= ((v == 0) as u8) << 1;
    }

    fn compare(&mut self, a: u8, b: u8) {
        let res = a - b;

        self.set_n(res);
        self.set_z(res);
        self.cpu.p &= 0xfe;
        self.cpu.p |= (b <= a) as u8;
    }

    pub(crate) fn step_everything(&mut self) {
        let inst = self.fetch_pc();
        let a = inst >> 5;
        let b = (inst >> 2) & 7;
        let c = inst & 3;

        match (a, b, c) {
            // c = 0 nops
            (0 | 2 | 3, 1, 0) => { addr_mode!(load self addr_of_zp); },
            (0, 3, 0) => { addr_mode!(load self addr_of_abs); },
            (0..=3 | 6..=7, 5, 0) => { addr_mode!(load self addr_of_zp_x); },
            (0..=3 | 6..=7, 7, 0) => { addr_mode!(load self addr_of_abs_x); },

            (1, 0, 0) => {
                self.push_u16(self.cpu.pc + 1);
                self.cpu.pc = self.addr_of_abs();
                self.elapse_cycles(1);
            },
            (1, 1, 0) => { // bit zp
                let m = addr_mode!(load self addr_of_zp);
                self.cpu.p &= 0x3f;
                self.cpu.p |= m & 0xc0;
                self.set_z(self.cpu.a & m);
            },
            (1, 3, 0) => { // bit abs
                let m = addr_mode!(load self addr_of_abs);
                self.cpu.p &= 0x3f;
                self.cpu.p |= m & 0xc0;
                self.set_z(self.cpu.a & m);
            },

            (0, 2, 0) => self.push(self.cpu.p | 0x10),
            (1, 2, 0) => {
                self.cpu.p = (self.pop() & 0xef) | 0x20;
                self.elapse_cycles(1);
            },
            (2, 2, 0) => self.push(self.cpu.a),
            (3, 2, 0) => {
                set_val_nz!(self self.cpu.a, = self.pop());
                self.elapse_cycles(1);
            },
            (4, 2, 0) => set_val_nz!(self self.cpu.y, -= 1),
            (5, 2, 0) => set_val_nz!(self self.cpu.y, = self.cpu.a),
            (6, 2, 0) => set_val_nz!(self self.cpu.y, += 1),
            (7, 2, 0) => set_val_nz!(self self.cpu.x, += 1),

            (0, 6, 0) => self.cpu.p &= 0xfe, // clc
            (1, 6, 0) => self.cpu.p |= 0x01, // sec
            (2, 6, 0) => self.cpu.p &= 0xfb, // cli
            (3, 6, 0) => self.cpu.p |= 0x04, // sei
            (4, 6, 0) => set_val_nz!(self self.cpu.a, = self.cpu.y),
            (5, 6, 0) => self.cpu.p &= 0xbf, // clv
            (6, 6, 0) => self.cpu.p &= 0xf7, // cld
            (7, 6, 0) => self.cpu.p |= 0x08, // sed

            (2, 3, 0) => self.cpu.pc = self.addr_of_abs(),
            (3, 3, 0) => {
                let ind = self.addr_of_abs();
                let l = self.load(ind);
                let h = self.load((ind & 0xff00) | ((ind + 1) & 0xff));
                self.cpu.pc = ((h as u16) << 8) | (l as u16);
            },
            (cond, 4, 0) => { // bxx
                let bit = (self.cpu.p >> match cond >> 1 {
                    0 => 7,
                    1 => 6,
                    2 => 0,
                    3 => 1,
                    _ => unreachable!()
                }) & 1;

                let inc = self.fetch_pc() as i8 as u16;
                if bit == cond & 1 {
                    self.elapse_cycles(1 + (self.cpu.pc >> 8 != (self.cpu.pc + inc) >> 8) as usize);
                    self.cpu.pc += inc as i8 as u16;
                }
            },

            (2, 0, 0) => { // rti
                self.cpu.p = self.pop() | 0x20;
                self.cpu.pc = self.pop_u16();
                self.elapse_cycles(1);
            },
            (3, 0, 0) => {
                self.cpu.pc = self.pop_u16() + 1;
                self.elapse_cycles(2);
            },

            (4, 0, 0) => { self.fetch_pc(); },
            (4, 1, 0) => addr_mode!(store self addr_of_zp self.cpu.y),
            (4, 3, 0) => addr_mode!(store self addr_of_abs self.cpu.y),
            (4, 5, 0) => addr_mode!(store self addr_of_zp_x self.cpu.y),

            (5, _, 0) => set_val_nz!(self self.cpu.y, = match b {
                0 => self.fetch_pc(),
                1 => addr_mode!(load self addr_of_zp),
                2 => self.cpu.a,
                3 => addr_mode!(load self addr_of_abs),
                5 => addr_mode!(load self addr_of_zp_x),
                7 => addr_mode!(load self addr_of_abs_x),
                _ => unreachable!(),
            }),
            (6 | 7, _, 0) => {
                let opr = match b {
                    0 => self.fetch_pc(),
                    1 => addr_mode!(load self addr_of_zp),
                    3 => addr_mode!(load self addr_of_abs),
                    5 => addr_mode!(load self addr_of_zp_x),
                    7 => addr_mode!(load self addr_of_abs_x),
                    _ => unreachable!(),
                };

                if b < 4 {
                    self.compare(if a == 6 { self.cpu.y } else { self.cpu.x }, opr);
                }
            },

            (4, 2, 1) => { self.fetch_pc(); }, // nop imm
            (4, 0, 1) => addr_mode!(store self addr_of_indx_indr self.cpu.a),
            (4, 1, 1) => addr_mode!(store self addr_of_zp self.cpu.a),
            (4, 3, 1) => addr_mode!(store self addr_of_abs self.cpu.a),
            (4, 4, 1) => addr_mode!(store self addr_of_indr_indx_store self.cpu.a),
            (4, 5, 1) => addr_mode!(store self addr_of_zp_x self.cpu.a),
            (4, 6, 1) => addr_mode!(store self addr_of_abs_y_store self.cpu.a),
            (4, 7, 1) => addr_mode!(store self addr_of_abs_x_store self.cpu.a),

            (_, _, 1) | (7, 2, 3) => {
                let opr = match b {
                    0 => addr_mode!(load self addr_of_indx_indr),
                    1 => addr_mode!(load self addr_of_zp),
                    2 => self.fetch_pc(),
                    3 => addr_mode!(load self addr_of_abs),
                    4 => addr_mode!(load self addr_of_indr_indx),
                    5 => addr_mode!(load self addr_of_zp_x),
                    6 => addr_mode!(load self addr_of_abs_y),
                    7 => addr_mode!(load self addr_of_abs_x),
                    _ => unreachable!(),
                };

                match a {
                    0 => self.cpu.a |= opr,
                    1 => self.cpu.a &= opr,
                    2 => self.cpu.a ^= opr,
                    5 => self.cpu.a = opr,
                    6 => self.compare(self.cpu.a, opr),
                    3 => {
                        let (a, c1) = self.cpu.a.overflowing_add(opr);
                        let (res, c2) = a.overflowing_add(self.cpu.p & 1);

                        self.cpu.p &= 0xbe;
                        self.cpu.p |= (c1 | c2) as u8;
                        self.cpu.p |= ((!(self.cpu.a ^ opr) & (self.cpu.a ^ res) & 0x80) >> 1) as u8;
                        self.cpu.a = res;
                    },
                    7 => {
                        let res = self.cpu.a as i8 as i16 - opr as i8 as i16 - (1 - (self.cpu.p & 1)) as i16;

                        self.cpu.p &= 0xbe;
                        self.cpu.p |= (res as i8 >= 0) as u8;
                        self.cpu.p |= ((res > 127 || res < -128) as u8) << 6;
                        self.cpu.a = res as u8;
                    },
                    _ => unreachable!(),
                }

                if a != 6 {
                    self.set_n(self.cpu.a);
                    self.set_z(self.cpu.a);
                }
            },

            (4 | 6 | 7, 0, 2) => { self.fetch_pc(); } // 2 byte nop
            (4, 1, 2) => addr_mode!(store self addr_of_zp self.cpu.x),
            (4, 2, 2) => set_val_nz!(self self.cpu.a, = self.cpu.x),
            (4, 3, 2) => addr_mode!(store self addr_of_abs self.cpu.x),
            (4, 5, 2) if a == 4 => addr_mode!(store self addr_of_zp_y self.cpu.x),
            (4, 5, 2) => addr_mode!(store self addr_of_zp_x self.cpu.x),
            (4, 6, 2) => self.cpu.s = self.cpu.x,
            (4, 7, 2) => todo!("shx"),

            (5, _, 2) => set_val_nz!(self self.cpu.x, = match b {
                0 => self.fetch_pc(),
                1 => addr_mode!(load self addr_of_zp),
                2 => self.cpu.a,
                3 => addr_mode!(load self addr_of_abs),
                4 => todo!("jam"),
                5 => addr_mode!(load self addr_of_zp_y),
                6 => self.cpu.s,
                7 => addr_mode!(load self addr_of_abs_y),
                _ => unreachable!(),
            }),

            (6, 2, 2) => set_val_nz!(self self.cpu.x, -= 1),
            (7, 2, 2) => {}, // 0xea nop
            (_, 6, 2) => {},

            (_, _, 2) => {
                let addr = match b {
                    0 | 4 => todo!("jam"),
                    1 => Some(self.addr_of_zp()),
                    2 => None,
                    3 => Some(self.addr_of_abs()),
                    5 => Some(self.addr_of_zp_x()),
                    7 => Some(self.addr_of_abs_x_store()),
                    _ => unreachable!(),
                };
                let m = addr.map_or(self.cpu.a, |addr| self.load(addr));

                let v = match a {
                    0 => {
                        self.cpu.p &= 0xfe;
                        self.cpu.p |= m >> 7;
                        m << 1
                    },
                    1 => {
                        let v = (m << 1) | (self.cpu.p & 1);
                        self.cpu.p &= 0xfe;
                        self.cpu.p |= m >> 7;
                        v
                    }
                    2 => {
                        self.cpu.p &= 0xfe;
                        self.cpu.p |= m & 1;
                        m >> 1
                    },
                    3 => {
                        let v = (self.cpu.p << 7) | (m >> 1);
                        self.cpu.p &= 0xfe;
                        self.cpu.p |= m & 1;
                        v
                    },
                    6 => m - 1,
                    7 => m + 1,
                    _ => unreachable!(),
                };

                self.set_n(v);
                self.set_z(v);

                if let Some(addr) = addr {
                    self.elapse_cycles(1);
                    self.store(addr, v);
                } else {
                    self.cpu.a = v;
                }
            },

            (5, _, 3) => set_val_nz!(self self.cpu.a, self.cpu.x,, match b {
                0 => addr_mode!(load self addr_of_indx_indr),
                1 => addr_mode!(load self addr_of_zp),
                2 => self.fetch_pc(),
                3 => addr_mode!(load self addr_of_abs),
                4 => addr_mode!(load self addr_of_indr_indx),
                5 => addr_mode!(load self addr_of_zp_y),
                // 6 => addr_mode!(load self addr_of_abs_y),
                7 => addr_mode!(load self addr_of_abs_y),
                _ => unreachable!(),
            }),

            // (4, 2, 3) => { self.fetch_pc(); }, // nop imm
            (4, 0, 3) => addr_mode!(store self addr_of_indx_indr self.cpu.a & self.cpu.x),
            (4, 1, 3) => addr_mode!(store self addr_of_zp self.cpu.a & self.cpu.x),
            (4, 3, 3) => addr_mode!(store self addr_of_abs self.cpu.a & self.cpu.x),
            // (4, 4, 3) => addr_mode!(store self addr_of_indr_indx self.cpu.a & self.cpu.x),
            (4, 5, 3) => addr_mode!(store self addr_of_zp_y self.cpu.a & self.cpu.x),
            // (4, 6, 3) => addr_mode!(store self addr_of_abs_y self.cpu.a & self.cpu.x),
            // (4, 7, 3) => addr_mode!(store self addr_of_abs_x self.cpu.a & self.cpu.x),

            (0, 0, 3) => slo!(self addr_of_indx_indr),
            (0, 1, 3) => slo!(self addr_of_zp),
            (0, 3, 3) => slo!(self addr_of_abs),
            (0, 4, 3) => slo!(self addr_of_indr_indx),
            (0, 5, 3) => slo!(self addr_of_zp_x),
            (0, 6, 3) => slo!(self addr_of_abs_y),
            (0, 7, 3) => slo!(self addr_of_abs_x),

            (1, 0, 3) => rla!(self addr_of_indx_indr),
            (1, 1, 3) => rla!(self addr_of_zp),
            (1, 3, 3) => rla!(self addr_of_abs),
            (1, 4, 3) => rla!(self addr_of_indr_indx),
            (1, 5, 3) => rla!(self addr_of_zp_x),
            (1, 6, 3) => rla!(self addr_of_abs_y),
            (1, 7, 3) => rla!(self addr_of_abs_x),

            (2, 0, 3) => sre!(self addr_of_indx_indr),
            (2, 1, 3) => sre!(self addr_of_zp),
            (2, 3, 3) => sre!(self addr_of_abs),
            (2, 4, 3) => sre!(self addr_of_indr_indx),
            (2, 5, 3) => sre!(self addr_of_zp_x),
            (2, 6, 3) => sre!(self addr_of_abs_y),
            (2, 7, 3) => sre!(self addr_of_abs_x),

            (3, 0, 3) => rra!(self addr_of_indx_indr),
            (3, 1, 3) => rra!(self addr_of_zp),
            (3, 3, 3) => rra!(self addr_of_abs),
            (3, 4, 3) => rra!(self addr_of_indr_indx),
            (3, 5, 3) => rra!(self addr_of_zp_x),
            (3, 6, 3) => rra!(self addr_of_abs_y),
            (3, 7, 3) => rra!(self addr_of_abs_x),

            (6, 0, 3) => dcp!(self addr_of_indx_indr),
            (6, 1, 3) => dcp!(self addr_of_zp),
            (6, 3, 3) => dcp!(self addr_of_abs),
            (6, 4, 3) => dcp!(self addr_of_indr_indx),
            (6, 5, 3) => dcp!(self addr_of_zp_x),
            (6, 6, 3) => dcp!(self addr_of_abs_y),
            (6, 7, 3) => dcp!(self addr_of_abs_x),

            (7, 0, 3) => isc!(self addr_of_indx_indr),
            (7, 1, 3) => isc!(self addr_of_zp),
            (7, 3, 3) => isc!(self addr_of_abs),
            (7, 4, 3) => isc!(self addr_of_indr_indx),
            (7, 5, 3) => isc!(self addr_of_zp_x),
            (7, 6, 3) => isc!(self addr_of_abs_y),
            (7, 7, 3) => isc!(self addr_of_abs_x),
            _ => todo!("{inst:02x} {a} {b} {c}"),
        }

        if core::mem::take(&mut self.fetched_bytes) == 1 {
            self.load(self.cpu.pc);
        }
    }

    fn fetch_pc(&mut self) -> u8 {
        let ret = self.load(self.cpu.pc);
        self.cpu.pc += 1;
        self.fetched_bytes += 1;
        ret
    }

    fn fetch_u16(&mut self) -> u16 {
        let ret = self.load_u16(self.cpu.pc);
        self.cpu.pc += 2;
        self.fetched_bytes += 2;
        ret
    }

    fn push(&mut self, val: u8) {
        self.store(0x0100 + self.cpu.s as u16, val);
        self.cpu.s -= 1;
    }

    fn push_u16(&mut self, val: u16) {
        self.store_u16(0x00ff + self.cpu.s as u16, val);
        self.cpu.s -= 2;
    }

    fn pop(&mut self) -> u8 {
        self.cpu.s += 1;
        self.load(0x0100 + self.cpu.s as u16)
    }

    fn pop_u16(&mut self) -> u16 {
        self.cpu.s += 2;
        self.load_u16(0x00ff + self.cpu.s as u16)
    }
}
