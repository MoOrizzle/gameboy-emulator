mod rom;
mod gameboy;

use std::time::{Duration, Instant};

use gameboy::{screen::window::ScreenWindow, GameBoy};


fn main() {
    let rom = rom::handle_rom();

    let mut gb = GameBoy::new(rom);
    let mut screen = ScreenWindow::new("MoBoy - Emulator", 4);

    let frame_duration = Duration::from_micros(16742); //~59.7 Hz
    let mut next_frame_time = Instant::now();

    while screen.is_open() {
        let inputs = screen.get_input();
        for (key, is_pressed) in inputs {
            if is_pressed {
                gb.key_down(key);
            } else {
                gb.key_up(key);
            }
        }

        if gb.step() {
            screen.draw(&gb.ppu.framebuffer);

            next_frame_time += frame_duration;
            let now = Instant::now();
            if next_frame_time > now {
                std::thread::sleep(next_frame_time - now);
            } else {
                next_frame_time = now;
            }
        }
    }
}