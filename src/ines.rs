use nes::cart::Cartridge;
use nes::ppu::CiRam;
use std::io;

pub struct InesFile<'a> {
    pub mapper_id: u16,
    pub submapper: u8,

    pub prg_rom: &'a [u8],
    pub chr_rom: &'a [u8],

    pub prg_ram_size: u16,
    pub chr_ram_size: u16,
    pub eeprom_size: u16,

    pub vert_mirror: bool,
    pub alt_nt_layout: bool,
}

impl<'a> InesFile<'a> {
    pub fn new(bytes: &'a [u8]) -> io::Result<Self> {
        if bytes.len() < 16 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "file too short"));
        }

        if &bytes[0..=3] != b"NES\x1a" {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "header incorrect"));
        }

        // in 16 kib
        let mut prg_rom_size = bytes[4] as usize;
        // in 8 kib
        let mut chr_rom_size = bytes[5] as usize;

        let vert_mirror = bytes[6] & 1 != 0;
        let header_end = 16 + (bytes[6] & 4 != 0) as usize * 512;
        let alt_nt_layout = bytes[6] & 8 != 0;

        let mut mapper_id = ((bytes[7] & 0xf0) as u16) | ((bytes[6] >> 4) as u16);
        let mut submapper = 0;

        if bytes[7] & 0x0c == 8 {
            // nes 2.0

            mapper_id |= ((bytes[8] & 0x0f) as u16) << 8;
            submapper = bytes[8] >> 4;

            prg_rom_size |= ((bytes[9] & 0x0f) as usize) << 8;
            chr_rom_size |= ((bytes[9] & 0xf0) as usize) << 4;

            // TODO: offset 10 & 11
        }

        let prg_rom_end = header_end + prg_rom_size * 16384;
        let chr_rom_end = prg_rom_end + chr_rom_size * 8192;

        Ok(Self {
            mapper_id,
            submapper,

            prg_rom: &bytes[header_end..prg_rom_end],
            chr_rom: &bytes[prg_rom_end..chr_rom_end],

            prg_ram_size: 0,
            chr_ram_size: 0,
            eeprom_size: 0,

            vert_mirror,
            alt_nt_layout,
        })
    }
}

macro_rules! mappers {
    ($($id:tt : $name:tt $(< $($lt:tt),+ >)?),* $(,)?) => {
        pub enum InesMapper<'a> {
            $($name($name $(< $($lt)* >)?)),*
        }

        impl<'a> InesMapper<'a> {
            pub fn new(file: InesFile<'a>) -> Self {
                match file.mapper_id {
                    $($id => Self::$name($name::new(file)),)*
                    _ => todo!("ines mapper id #{:03x}", file.mapper_id),
                }
            }
        }

        impl Cartridge for InesMapper<'_> {
            fn load(&mut self, addr: u16) -> Result<u8, ()> {
                match self {
                    $(Self::$name(m) => m.load(addr)),*
                }
            }

            fn store(&mut self, addr: u16, data: u8) -> Result<(), ()> {
                match self {
                    $(Self::$name(m) => m.store(addr, data)),*
                }
            }

            fn vmem_load(&mut self, ciram: &CiRam, addr: u16) -> u8 {
                match self {
                    $(Self::$name(m) => m.vmem_load(ciram, addr)),*
                }
            }

            fn vmem_store(&mut self, ciram: &mut CiRam, addr: u16, data: u8) {
                match self {
                    $(Self::$name(m) => m.vmem_store(ciram, addr, data)),*
                }
            }
        }
    };
}

fn vert_mirror(addr: u16) -> u16 {
    addr
}

fn horiz_mirror(addr: u16) -> u16 {
    ((addr & 0xf800) >> 1) | (addr & 0x7ff)
}

mappers!(
    0x000: Nrom<'a>,
);

/// INES mapper 000
pub struct Nrom<'a> {
    prg_rom: &'a [u8],
    prg_rom_mask: u16,

    prg_ram: Box<[u8]>,
    prg_ram_mask: u16,

    chr_rom: &'a [u8],
    vert_mirror: bool,
}

impl<'a> Nrom<'a> {
    pub fn new(file: InesFile<'a>) -> Self {
        Self {
            prg_rom: file.prg_rom,
            prg_rom_mask: file.prg_rom.len() as u16 - 1,

            prg_ram: vec![0; file.prg_ram_size as usize].into(),
            prg_ram_mask: file.prg_ram_size - 1,

            chr_rom: file.chr_rom,
            vert_mirror: file.vert_mirror,
        }
    }
}

impl Cartridge for Nrom<'_> {
    fn load(&mut self, addr: u16) -> Result<u8, ()> {
        match addr {
            0x6000..=0x7fff => Ok(self.prg_ram[(addr & self.prg_ram_mask) as usize]),
            0x8000..=0xffff => Ok(self.prg_rom[(addr & self.prg_rom_mask) as usize]),
            _ => Err(()),
        }
    }

    fn store(&mut self, addr: u16, data: u8) -> Result<(), ()> {
        match addr {
            0x6000..=0x7fff => Ok(self.prg_ram[(addr & self.prg_ram_mask) as usize] = data),
            _ => Err(()),
        }
    }

    fn vmem_load(&mut self, ciram: &nes::ppu::CiRam, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1fff => self.chr_rom[addr as usize],
            0x2000..=0x2fff if self.vert_mirror => ciram[(vert_mirror(addr) & 0x7ff) as usize],
            0x2000..=0x2fff if !self.vert_mirror => ciram[(horiz_mirror(addr) & 0x7ff) as usize],
            _ => addr as u8,
        }
    }

    fn vmem_store(&mut self, ciram: &mut nes::ppu::CiRam, addr: u16, data: u8) {
        match addr {
            0x2000..=0x2fff if self.vert_mirror => ciram[(vert_mirror(addr) & 0x7ff) as usize] = data,
            0x2000..=0x2fff if !self.vert_mirror => ciram[(horiz_mirror(addr) & 0x7ff) as usize] = data,
            _ => {},
        }
    }
}
