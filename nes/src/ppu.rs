use super::*;

#[derive(Debug, Clone)]
pub(crate) struct Ppu {
    pub scanline: usize,
    pub cycle: usize,

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
        self.ppu.scanline = (self.ppu.scanline + 1) % 262;
        self.ppu.cycle = (self.ppu.cycle + 1) % 341;

        if self.ppu.frame_odd && self.ppu.scanline == 261 && self.ppu.cycle == 339 {
            self.ppu.scanline = 0;
            self.ppu.cycle = 0;
        }

        self.ppu.frame_odd ^= self.ppu.scanline == 0 && self.ppu.cycle == 0;

        match (self.ppu.scanline, self.ppu.cycle) {
            (0..=239 | 261, 0) => {}, // idle cycle
            (0..=239 | 261, 1..=256) => { // tile fetch
            },
            (0..=239 | 261, 257..=320) => { // next scanline sprite fetch
            },
            (0..=239 | 261, 321..=336) => { // next scanline first 2 tile fetch
            },
            (0..=239 | 261, 337 | 339) => { // dummy fetch next scanline tile 3
            },
            (0..=239 | 261, 337..=340) => {}, // fetching
            (241, 0) => { // vblank stuff
            },
            (240..=260, _) => {}, // idle
            _ => unreachable!(),
        }
    }
}
