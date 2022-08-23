mod nes;
mod bus;
mod cpu;
mod cartridge;

use nes::NES;

fn main() {
    let nes = NES::new();
    nes.powerup();
}
