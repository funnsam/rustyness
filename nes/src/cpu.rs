use super::*;

macro_rules! addr_mode {
    ($self: tt $name: ident) => {{
        let addr = $self.$name();
        $self.load(addr)
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

    pub(crate) fn step_everything(&mut self) {
        let inst = self.fetch_pc();
        let a = inst >> 5;
        let b = (inst >> 2) & 7;
        let c = inst & 3;

        match c {
            0 => match (a, b) {
                (2, 3) => { // jmp abs
                    let addr = self.addr_of_abs();
                    self.cpu.pc = addr;
                },
                _ => todo!("{a} {b} {c}"),
            },
            1 => {
                if a == 4 { // sta
                    todo!("sta {b}");
                } else {
                    let opr = match b {
                        0 => addr_mode!(self addr_of_indx_indr),
                        1 => addr_mode!(self addr_of_zp),
                        2 => self.fetch_pc(),
                        3 => addr_mode!(self addr_of_abs),
                        4 => addr_mode!(self addr_of_indr_indx),
                        5 => addr_mode!(self addr_of_zp_x),
                        6 => addr_mode!(self addr_of_abs_y),
                        7 => addr_mode!(self addr_of_abs_x),
                        _ => unreachable!(),
                    };

                    match a {
                        0 => todo!("ora"),
                        1 => todo!("and"),
                        2 => todo!("eor"),
                        3 => todo!("adc"),
                        5 => self.cpu.a = opr, // lda
                        6 => todo!("cmp"),
                        7 => todo!("sbc"),
                        _ => unreachable!(),
                    }
                }
            },
            2 => todo!("{a} {b} {c}"),
            3 => todo!("{a} {b} {c}"),
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
}
