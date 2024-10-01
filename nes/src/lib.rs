mod cart;
mod cpu;
mod ppu;

#[cfg(test)]
mod test;

pub struct Nes<'a> {
    cpu: cpu::Cpu,
    ppu: ppu::Ppu,

    iram: [u8; 0x800],
    cart: &'a mut dyn cart::Cartridge,

    last_read: u8,
    cycles_ahead: usize,
    fetched_bytes: usize,
}

impl<'a> Nes<'a> {
    pub fn new(cart: &'a mut dyn cart::Cartridge, start: Option<u16>) -> Self {
        let fffc = cart.load(0xfffc).unwrap();
        let fffd = cart.load(0xfffd).unwrap();

        Self {
            cpu: cpu::Cpu::new(start, fffc, fffd),
            ppu: ppu::Ppu::new(),

            iram: [0; 0x800],
            cart,

            last_read: 0,
            cycles_ahead: 7,
            fetched_bytes: 0,
        }
    }

    pub fn step(&mut self) {
        self.cycles_ahead -= 1;

        if self.cycles_ahead != 0 {
            return;
        }

        self.step_everything();
    }

    fn step_not_cpu(&mut self) {
        for _ in 0..3 { self.step_ppu(); }
    }

    fn elapse_cycles(&mut self, cy: usize) {
        self.cycles_ahead += cy;

        for _ in 0..cy {
            self.step_not_cpu();
        }
    }

    fn load(&mut self, addr: u16) -> u8 {
        self.elapse_cycles(1);

        if let Ok(v) = self._load(addr) {
            self.last_read = v;
            v
        } else {
            self.last_read
        }
    }

    fn load_u16(&mut self, addr: u16) -> u16 {
        let l = self.load(addr);
        let h = self.load(addr + 1);
        ((h as u16) << 8) | (l as u16)
    }

    fn _load(&mut self, addr: u16) -> Result<u8, ()> {
        match addr {
            0x0000..=0x1fff => Ok(self.iram[addr as usize & 0x7ff]),
            0x2000..=0x3fff => Err(()), // PPU regs
            0x4000..=0x4017 => Err(()), // APU & IO
            0x4018..=0x401f => Err(()), // APU & IO test mode
            0x4020..=0xffff => self.cart.load(addr),
        }
    }

    fn store(&mut self, addr: u16, val: u8) {
        self.elapse_cycles(1);

        _ = self._store(addr, val);
    }

    fn store_u16(&mut self, addr: u16, val: u16) {
        self.store(addr, val as u8);
        self.store(addr + 1, (val >> 8) as u8);
    }

    fn _store(&mut self, addr: u16, val: u8) -> Result<(), ()> {
        match addr {
            0x0000..=0x1fff => Ok(self.iram[addr as usize & 0x7ff] = val),
            0x2000..=0x3fff => Err(()), // PPU regs
            0x4000..=0x4017 => Err(()), // APU & IO
            0x4018..=0x401f => Err(()), // APU & IO test mode
            0x4020..=0xffff => self.cart.store(addr, val),
        }
    }
}
