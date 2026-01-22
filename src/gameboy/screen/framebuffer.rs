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
            pixels: [[0xFF000000; SCREEN_W]; SCREEN_H],
        }
    }

    pub fn clear(&mut self, color: Color) {
        for y in 0..SCREEN_H {
            for x in 0..SCREEN_W {
                self.pixels[y][x] = color;
            }
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

    pub fn draw_test_pattern(&mut self) {
        for y in 0..SCREEN_H {
            for x in 0..SCREEN_W {
                let shade = ((((x / 8) + (y / 8)) % 4) * 85) as u32;
                self.pixels[y][x] =
                    0xFF000000 | (shade << 16) | (shade << 8) | shade;
            }
        }
    }
}