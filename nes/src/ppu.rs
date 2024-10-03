use super::*;

#[derive(Debug, Clone)]
pub struct Ppu {
    pub scanline: usize,
    pub cycle: usize,

    pub ciram: CiRam,

    /// Right shifted 8 bits
    pub base_nt: u8,
    pub ppudata_inc: u8,
    pub sp_pattern: bool,
    pub bg_pattern: bool,
    pub large_sprite: bool,
    pub nmi_on_vblank: bool,

    pub grayscale: bool,
    pub bg_show_left: bool,
    pub sp_show_left: bool,
    pub show_bg: bool,
    pub show_sp: bool,
    // TODO: emphasize colors

    pub sp_overflow: bool,
    pub sp0_hit: bool,
    pub vblank_flag: bool,

    pub scroll: [u8; 2],
    pub addr_status: u16,

    /// current VRAM address
    v: u16,
    /// temp VRAM address (top left on-screen tile)
    t: u16,
    /// fine x scroll
    x: u8,
    /// write toggle
    w: bool,

    /// odd frame toggle
    /// https://www.nesdev.org/wiki/PPU_frame_timing#Even/Odd_Frames
    frame_odd: bool,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            scanline: 0,
            cycle: 21,

            ciram: [0; 2048],

            base_nt: 0,
            ppudata_inc: 0,
            sp_pattern: false,
            bg_pattern: false,
            large_sprite: false,
            nmi_on_vblank: false,

            grayscale: false,
            bg_show_left: false,
            sp_show_left: false,
            show_bg: false,
            show_sp: false,

            sp_overflow: false,
            sp0_hit: false,
            vblank_flag: false,

            scroll: [0; 2],
            addr_status: 0,

            v: 0,
            t: 0,
            x: 0,
            w: false,

            frame_odd: false,
        }
    }
}

pub type CiRam = [u8; 2048];

impl Nes<'_> {
    pub(crate) fn step_ppu(&mut self) {
        self.ppu.scanline += (self.ppu.cycle == 340) as usize;
        self.ppu.cycle = (self.ppu.cycle + 1) % 341;

        if (self.ppu.frame_odd && self.ppu.scanline == 261 && self.ppu.cycle == 339) || self.ppu.scanline == 262 {
            self.ppu.scanline = 0;
            self.ppu.cycle = 0;

            self.ppu.frame_odd ^= true;
        }

        match (self.ppu.scanline, self.ppu.cycle) {
            (0..=239 | 261, 0) => {}, // idle cycle
            (0..=239 | 261, 1..=256) => { // tile fetch
                self.ppu.vblank_flag = false;
            },
            (0..=239 | 261, 257..=320) => { // next scanline sprite fetch
            },
            (0..=239 | 261, 321..=336) => { // next scanline first 2 tile fetch
            },
            (0..=239 | 261, 337 | 339) => { // dummy fetch next scanline tile 3
            },
            (0..=239 | 261, 337..=340) => {}, // fetching
            (241, 1) => { // vblank stuff
                self.ppu.vblank_flag = true;
            },
            (240..=260, _) => {}, // idle
            _ => unreachable!(),
        }
    }

    pub(crate) fn store_ppu_mmio(&mut self, addr: u16, data: u8) {
        match addr {
            0x2000 => {
                self.ppu.base_nt = [0x20, 0x24, 0x28, 0x2c][data as usize & 3];
                self.ppu.ppudata_inc = [1, 32][((data & 4) >> 2) as usize];
                self.ppu.sp_pattern = data & 0x08 != 0;
                self.ppu.bg_pattern = data & 0x10 != 0;
                self.ppu.large_sprite = data & 0x20 != 0;
                self.ppu.nmi_on_vblank = data & 0x80 != 0;
            },
            0x2001 => {
                self.ppu.grayscale = data & 0x01 != 0;
                self.ppu.bg_show_left = data & 0x02 != 0;
                self.ppu.sp_show_left = data & 0x04 != 0;
                self.ppu.show_bg = data & 0x08 != 0;
                self.ppu.show_sp = data & 0x10 != 0;
            },
            // TODO: oam addr & data
            0x2005 => {
                self.ppu.scroll[self.ppu.w as usize] = data;
                self.ppu.w ^= true;
            },
            0x2006 => {
                if self.ppu.w {
                    // TODO: bus conflict causing vertical scrolling weird things
                }

                if !self.ppu.w {
                    // high
                    self.ppu.addr_status &= 0x00ff;
                    self.ppu.addr_status |= (data as u16) << 8;
                } else {
                    // low
                    self.ppu.addr_status &= 0xff00;
                    self.ppu.addr_status |= data as u16;
                }

                self.ppu.w ^= true;
            },
            0x2007 => {
                self.cart.vmem_store(&mut self.ppu.ciram, self.ppu.addr_status, data);
                self.ppu.addr_status += self.ppu.ppudata_inc as u16;
            },
            // TODO: oam dma
            _ => {},
        }
    }

    pub(crate) fn load_ppu_mmio(&mut self, addr: u16) -> Result<u8, ()> {
        match addr {
            0x2002 => {
                let r = ((self.ppu.sp_overflow as u8) << 5) | ((self.ppu.sp0_hit as u8) << 6) | ((self.ppu.vblank_flag as u8) << 7);
                self.ppu.vblank_flag = false;
                self.ppu.w = false;
                Ok(r)
            },
            0x2007 => {
                let r = self.cart.vmem_load(&mut self.ppu.ciram, self.ppu.addr_status);
                self.ppu.addr_status += self.ppu.ppudata_inc as u16;
                Ok(r)
            },
            _ => Err(())
        }
    }
}
