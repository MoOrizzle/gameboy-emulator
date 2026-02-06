mod rom;
mod gameboy;

use gameboy::{screen::window::ScreenWindow, GameBoy, audio::AudioQueue};


fn main() {
    let rom = rom::handle_rom();

    let mut gb = GameBoy::new(rom);
    let mut screen = ScreenWindow::new("MoBoy - Emulator", 4);
    let audio = AudioQueue::new();

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
        }

        let (left, right) = gb.get_audio_samples();
        if !left.is_empty() {
            audio.push_samples(&left, &right);
        }
    }
}