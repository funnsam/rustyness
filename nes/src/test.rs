use std::io::BufRead;

use super::*;

#[test]
fn run_nestest() {
    // reference: https://www.qmtpro.com/~nes/misc/nestest.txt
    // for rom and log check makefile

    struct TestCart<'a>(&'a [u8]);

    impl cart::Cartridge for TestCart<'_> {
        fn load(&mut self, addr: u16) -> Result<u8, ()> {
            Ok(self.0[(addr as usize - 0x8000) & 16383])
        }

        fn store(&mut self, _addr: u16, _data: u8) -> Result<(), ()> { Err(()) }
    }

    let rom = std::fs::read("../tests/nestest.nes").unwrap();
    let mut cart = TestCart(&rom[16..16 + 16384]);
    let mut nes = Nes::new(&mut cart, Some(0xc000));

    let mut ref_log = std::io::BufReader::new(std::fs::File::open("../tests/nestest.log").unwrap());
    let mut log = String::new();

    loop {
        log.clear();
        ref_log.read_line(&mut log).unwrap();
        let mut line = log.split_whitespace();
        let pc = u16::from_str_radix(line.next().unwrap(), 16).unwrap();
        let a = u8::from_str_radix(line.next().unwrap(), 16).unwrap();
        let x = u8::from_str_radix(line.next().unwrap(), 16).unwrap();
        let y = u8::from_str_radix(line.next().unwrap(), 16).unwrap();
        let p = u8::from_str_radix(line.next().unwrap(), 16).unwrap();
        let s = u8::from_str_radix(line.next().unwrap(), 16).unwrap();
        let cy = line.next().unwrap().parse::<usize>().unwrap();

        assert_eq!(pc, nes.cpu.pc, "pc on cycle {cy}");
        assert_eq!(a, nes.cpu.a, "a on cycle {cy}");
        assert_eq!(x, nes.cpu.x, "x on cycle {cy}");
        assert_eq!(y, nes.cpu.y, "y on cycle {cy}");
        assert_eq!(p, nes.cpu.p, "p on cycle {cy}");
        assert_eq!(s, nes.cpu.s, "s on cycle {cy}");

        nes.step();
    }
}
