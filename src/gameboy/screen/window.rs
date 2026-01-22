use minifb::{Key, Window, WindowOptions};

use super::framebuffer::{Framebuffer, SCREEN_H, SCREEN_W};

pub struct ScreenWindow {
    window: Window,
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

        let window = Window::new(
            title,
            SCREEN_W,
            SCREEN_H,
            options,
        )
        .expect("Failed to create window");

        Self { window }
    }

    pub fn is_open(&self) -> bool {
        self.window.is_open() && !self.window.is_key_down(Key::Escape)
    }

    pub fn draw(&mut self, framebuffer: &Framebuffer) {
        let buffer = framebuffer.as_flat_buffer();
        self.window
            .update_with_buffer(&buffer, SCREEN_W, SCREEN_H)
            .expect("Failed to update window");
    }
}