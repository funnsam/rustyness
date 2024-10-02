mod ines;

fn main() {
    let file = std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    let ines = ines::InesFile::new(&file).unwrap();

    println!("{}", ines.mapper_id);
    println!("{}", ines.submapper);
    println!("{}", ines.prg_ram_size);
    println!("{}", ines.chr_ram_size);
    println!("{}", ines.eeprom_size);
    println!("{}", ines.vert_mirror);
    println!("{}", ines.alt_nt_layout);
}
