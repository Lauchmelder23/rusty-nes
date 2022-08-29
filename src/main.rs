mod nes;
mod bus;
mod cpu;
mod instructions;
mod addressing;
mod cartridge;
mod mnemonic;

use nes::NES;

fn main() {
    let nes = NES::new();
    nes.powerup();
}
