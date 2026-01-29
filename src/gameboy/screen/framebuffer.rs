pub const SCREEN_W: usize = 160;
pub const SCREEN_H: usize = 144;

pub type Color = u32;

#[derive(Clone)]
pub struct Framebuffer {
    pub pixels: [[Color; SCREEN_W]; SCREEN_H],
}

impl Framebuffer {
    pub fn new() -> Self {
        Self {
            pixels: [[0xFFFFFFFF; SCREEN_W]; SCREEN_H],
        }
    }

    pub fn as_flat_buffer(&self) -> Vec<u32> {
        let mut out = Vec::with_capacity(SCREEN_W * SCREEN_H);
        for y in 0..SCREEN_H {
            for x in 0..SCREEN_W {
                out.push(self.pixels[y][x]);
            }
        }
        out
    }
}