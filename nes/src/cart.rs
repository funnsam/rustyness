pub trait Cartridge {
    /// Tries to load a `u8` from cartridge PRGR*M
    fn load(&mut self, addr: u16) -> Result<u8, ()>;
    fn store(&mut self, addr: u16, data: u8) -> Result<(), ()>;

    /// Load a `u8` from video memory, return low byte of `addr` on open bus
    fn vmem_load(&mut self, ciram: &crate::ppu::CiRam, addr: u16) -> u8;
    fn vmem_store(&mut self, ciram: &mut crate::ppu::CiRam, addr: u16, data: u8);
}
