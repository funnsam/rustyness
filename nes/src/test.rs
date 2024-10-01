use std::io::BufRead;

use super::*;

macro_rules! assert_eq_hex {
    ($a: expr, $b: expr, $($m: tt)*) => {{
        let a = $a;
        let b = $b;
        assert!(a == b, "assertion `left == right` failed: {0}\n  left: 0x{1:03$x}\n right: 0x{2:03$x}", format!($($m)*), a, b, core::mem::size_of_val(&a) * 2);
    }};
}

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

        if log.is_empty() { break; }

        let mut line = log.split_whitespace();
        let pc = u16::from_str_radix(line.next().unwrap(), 16).unwrap();
        let a = u8::from_str_radix(line.next().unwrap(), 16).unwrap();
        let x = u8::from_str_radix(line.next().unwrap(), 16).unwrap();
        let y = u8::from_str_radix(line.next().unwrap(), 16).unwrap();
        let p = u8::from_str_radix(line.next().unwrap(), 16).unwrap();
        let s = u8::from_str_radix(line.next().unwrap(), 16).unwrap();
        let cy = line.next().unwrap().parse::<usize>().unwrap();

        assert_eq_hex!(a, nes.cpu.a, "a on cycle {cy}");
        assert_eq_hex!(x, nes.cpu.x, "x on cycle {cy}");
        assert_eq_hex!(y, nes.cpu.y, "y on cycle {cy}");
        assert_eq_hex!(p, nes.cpu.p, "p on cycle {cy}");
        assert_eq_hex!(s, nes.cpu.s, "s on cycle {cy}");
        assert_eq_hex!(pc, nes.cpu.pc, "pc on cycle {cy}");
        assert_eq!(cy, nes.cycles_ahead, "cycle count");

        nes.step_everything();
    }
}
