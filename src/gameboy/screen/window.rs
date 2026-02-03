use minifb::{Key, Window, WindowOptions};

use super::framebuffer::{Framebuffer, SCREEN_H, SCREEN_W};
use crate::gameboy::joypad::Key as JoypadKey;

pub struct ScreenWindow {
    window: Window,
    prev_keys: [bool; 8]
}

impl ScreenWindow {
    pub fn new(title: &str, scale: usize) -> Self {
        let mut options = WindowOptions::default();
        options.resize = false;
        options.scale = match scale {
            1 => minifb::Scale::X1,
            2 => minifb::Scale::X2,
            4 => minifb::Scale::X4,
            _ => minifb::Scale::X1,
        };

        let window = Window::new(title, SCREEN_W, SCREEN_H, options)
            .expect("Failed to create window");

        Self { 
            window,
            prev_keys: [false; 8], 
        }
    }

    pub fn is_open(&self) -> bool {
        self.window.is_open() && !self.window.is_key_down(Key::Escape)
    }

    pub fn draw(&mut self, framebuffer: &Framebuffer) {
        let buffer = framebuffer.as_flat_buffer();
        self.window.update_with_buffer(&buffer, SCREEN_W, SCREEN_H)
            .expect("Failed to update window");
    }

    pub fn get_input(&mut self) -> Vec<(JoypadKey, bool)> {
        let mut inputs = Vec::new();

        let current = [
            self.window.is_key_down(Key::Right),
            self.window.is_key_down(Key::Left),
            self.window.is_key_down(Key::Up),
            self.window.is_key_down(Key::Down),
            self.window.is_key_down(Key::Z),
            self.window.is_key_down(Key::X),
            self.window.is_key_down(Key::Space),
            self.window.is_key_down(Key::S),
        ];

        let keys = [
            JoypadKey::Right,
            JoypadKey::Left,
            JoypadKey::Up,
            JoypadKey::Down,
            JoypadKey::A,
            JoypadKey::B,
            JoypadKey::Start,
            JoypadKey::Select,
        ];

        for i in 0..8 {
            if current[i] != self.prev_keys[i] {
                inputs.push((keys[i], current[i]));
            }
        }
        
        self.prev_keys = current;
        inputs
    }
}