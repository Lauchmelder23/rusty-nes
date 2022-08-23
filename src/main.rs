mod nes;
mod bus;
mod cpu;

use nes::NES;

fn main() {
    let nes = NES::new();
    nes.powerup();
}
