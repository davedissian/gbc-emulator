#![allow(dead_code)]

mod cpu;
mod machine;
mod memory;
mod cartridge;

fn main() {
    let mut device = machine::Machine::new();
    device.load_cartridge("roms/pokemon-gold.gbc");
    device.tick();
}
