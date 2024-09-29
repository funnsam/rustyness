use super::*;

macro_rules! addr_mode {
    (load $self: tt $name: ident) => {{
        let addr = $self.$name();
        $self.load(addr)
    }};
    (store $self: tt $name: ident $val: expr) => {{
        let addr = $self.$name();
        $self.store(addr, $val)
    }};
}

impl Nes<'_> {
    // aka (zp, x)
    fn addr_of_indx_indr(&mut self) -> u16 {
        let ind = self.fetch_pc() + self.cpu.x;
        self.load(ind as u16) as u16 | (self.load((ind + 1) as u16) as u16) << 8
    }

    fn addr_of_zp(&mut self) -> u16 {
        self.fetch_pc() as u16
    }

    fn addr_of_abs(&mut self) -> u16 {
        self.fetch_u16()
    }

    // aka (zp), y
    fn addr_of_indr_indx(&mut self) -> u16 {
        let ind = self.fetch_pc();
        (self.load(ind as u16) as u16 | (self.load((ind + 1) as u16) as u16) << 8) + self.cpu.y as u16
    }

    fn addr_of_zp_x(&mut self) -> u16 {
        let off = self.fetch_pc();
        (off + self.cpu.x) as u16
    }

    fn addr_of_zp_y(&mut self) -> u16 {
        let off = self.fetch_pc();
        (off + self.cpu.y) as u16
    }

    fn addr_of_abs_x(&mut self) -> u16 {
        let off = self.fetch_u16();
        off + self.cpu.x as u16
    }

    fn addr_of_abs_y(&mut self) -> u16 {
        let off = self.fetch_u16();
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

    pub(crate) fn step_everything(&mut self) {
        let inst = self.fetch_pc();
        let a = inst >> 5;
        let b = (inst >> 2) & 7;
        let c = inst & 3;

        match (a, b, c) {
            (1, 0, 0) => {
                self.push_u16(self.cpu.pc + 2);
                self.cpu.pc = self.addr_of_abs();
            },
            (1, 1, 0) => { // bit zp
                let m = self.cpu.a & addr_mode!(load self addr_of_zp);
                self.cpu.p &= 0x3f;
                self.cpu.p |= m & 0xc0;
                self.set_z(m);
            },
            (1, 2, 0) => { // bit abs
                let m = self.cpu.a & addr_mode!(load self addr_of_abs);
                self.cpu.p &= 0x3f;
                self.cpu.p |= m & 0xc0;
                self.set_z(m);
            },
            (0, 2, 0) => self.push(self.cpu.p | 0x10),
            (1, 2, 0) => self.cpu.p = self.pop(),
            (2, 2, 0) => self.push(self.cpu.a),
            (3, 2, 0) => {
                self.cpu.a = self.pop();
                self.set_n(self.cpu.a);
                self.set_z(self.cpu.a);
            },
            (0, 6, 0) => self.cpu.p &= 0xfe, // clc
            (1, 6, 0) => self.cpu.p |= 0x01, // sec
            (2, 6, 0) => self.cpu.p &= 0xfb, // cli
            (3, 6, 0) => self.cpu.p |= 0x04, // sei
            (5, 6, 0) => self.cpu.p &= 0xbf, // clv
            (6, 6, 0) => self.cpu.p &= 0xf7, // cld
            (7, 6, 0) => self.cpu.p |= 0x08, // sed
            (2, 3, 0) => {
                let addr = self.addr_of_abs();
                self.cpu.pc = addr;
            },
            (cond, 4, 0) => { // bxx
                let bit = (self.cpu.p >> match cond >> 1 {
                    0 => 7,
                    1 => 6,
                    2 => 0,
                    3 => 1,
                    _ => unreachable!()
                }) & 1;

                let inc = self.fetch_pc();
                if bit == cond & 1 {
                    self.cpu.pc += inc as u16;
                }
            },
            (3, 0, 0) => self.cpu.pc = self.pop_u16(),
            (_, _, 0) => todo!("{a} {b} {c}"),

            (4, 2, 1) => { self.fetch_pc(); }, // nop imm
            (4, 0, 1) => addr_mode!(store self addr_of_indx_indr self.cpu.a),
            (4, 1, 1) => addr_mode!(store self addr_of_zp self.cpu.a),
            (4, 3, 1) => addr_mode!(store self addr_of_abs self.cpu.a),
            (4, 4, 1) => addr_mode!(store self addr_of_indr_indx self.cpu.a),
            (4, 5, 1) => addr_mode!(store self addr_of_zp_x self.cpu.a),
            (4, 6, 1) => addr_mode!(store self addr_of_abs_y self.cpu.a),
            (4, 7, 1) => addr_mode!(store self addr_of_abs_x self.cpu.a),
            (_, _, 1) => {
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
                    0 => todo!("ora"),
                    1 => todo!("and"),
                    2 => todo!("eor"),
                    3 => todo!("adc"),
                    5 => { // lda
                        self.cpu.a = opr;
                        self.set_n(self.cpu.a);
                        self.set_z(self.cpu.a);
                    },
                    6 => todo!("cmp"),
                    7 => todo!("sbc"),
                    _ => unreachable!(),
                }
            },

            (5, _, 2) => {
                self.cpu.x = match b {
                    0 => self.fetch_pc(),
                    1 => addr_mode!(load self addr_of_zp),
                    2 => self.cpu.a,
                    3 => addr_mode!(load self addr_of_abs),
                    4 => todo!(),
                    5 => addr_mode!(load self addr_of_zp_y),
                    6 => todo!(),
                    7 => addr_mode!(load self addr_of_abs_y),
                    _ => unreachable!(),
                };

                self.set_n(self.cpu.x);
                self.set_z(self.cpu.x);
            },
            (4, 2, 2) => {
                self.cpu.a = self.cpu.x;
                self.set_n(self.cpu.a);
                self.set_z(self.cpu.a);
            },
            (4, 1, 2) => addr_mode!(store self addr_of_zp self.cpu.x),
            (6, 2, 2) => todo!("dex"),
            (7, 2, 2) => {}, // 0xea nop
            (_, _, 2) => todo!("{a} {b} {c}"),

            (_, _, 3) => todo!("{a} {b} {c}"),
            _ => unreachable!(),
        }
    }

    fn fetch_pc(&mut self) -> u8 {
        let ret = self.load(self.cpu.pc);
        self.cpu.pc += 1;
        ret
    }

    fn fetch_u16(&mut self) -> u16 {
        let ret = self.load_u16(self.cpu.pc);
        self.cpu.pc += 2;
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
