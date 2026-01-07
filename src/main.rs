mod rom;
mod gameboy;

use gameboy::GameBoy;

fn main() {
    let rom = rom::handle_rom();

    let mut gb = GameBoy::new(rom);

    for _ in 0..1_000_000 {
        gb.step();
    }
}

