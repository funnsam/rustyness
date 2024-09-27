pub trait Cartridge: core::fmt::Debug {
    fn load(&mut self, addr: u16) -> Result<u8, ()>;
    fn store(&mut self, addr: u16, data: u8) -> Result<(), ()>;
}
