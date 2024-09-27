use super::*;

impl Nes<'_> {
    pub(crate) fn step_everything(&mut self) {
        let inst = self.fetch_pc();
        let a = inst >> 5;
        let b = (inst >> 2) & 7;
        let c = inst & 3;

        match c {
            0 => todo!(),
            1 => {
                if a == 4 { // sta
                    todo!();
                } else {
                    let opr = match b {
                        0 => {
                            let ind = self.fetch_pc() + self.cpu.x;
                            let addr = self.load(ind as u16) as u16 | (self.load((ind + 1) as u16) as u16) << 8;
                            self.load(addr)
                        },
                        1 => {
                            let addr = self.fetch_pc() as u16;
                            self.load(addr)
                        },
                        2 => self.fetch_pc(),
                        3 => {
                            let addr = self.fetch_u16();
                            self.load(addr)
                        },
                        4 => {
                            let ind = self.fetch_pc();
                            let addr = (self.load(ind as u16) as u16 | (self.load((ind + 1) as u16) as u16) << 8) + self.cpu.y as u16;
                            self.load(addr)
                        },
                        5 => {
                            let off = self.fetch_pc();
                            self.load((off + self.cpu.x) as u16)
                        },
                        6 => {
                            let addr = self.fetch_u16() + self.cpu.y as u16;
                            self.load(addr)
                        },
                        7 => {
                            let addr = self.fetch_u16() + self.cpu.x as u16;
                            self.load(addr)
                        },
                        _ => unreachable!(),
                    };

                    match a {
                        0 => todo!(), // ora
                        1 => todo!(), // and
                        2 => todo!(), // eor
                        3 => todo!(), // adc
                        5 => self.cpu.a = b, // lda
                        6 => todo!(), // cmp
                        7 => todo!(), // sbc
                        _ => unreachable!(),
                    }
                }
            },
            2 => todo!(),
            3 => todo!(),
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
