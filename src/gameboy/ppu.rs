use super::{cpu::Interrupt, screen::framebuffer::Framebuffer, mmu::Mmu};

#[derive(PartialEq)]
pub enum PpuMode {
    OamScan = 0b10,
    Drawing = 0b11,
    HBlank = 0b00,
    VBlank = 0b01,
}

pub struct Ppu {
    framebuffer: Framebuffer,
    mode: PpuMode,
    scanline: u8,
    pub frame_ready: bool,
    dot_counter: u16
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            framebuffer: Framebuffer::new(),
            mode: PpuMode::OamScan,
            scanline: 0,
            frame_ready: false,
            dot_counter: 0,
        }
    }

    pub fn step(&mut self, cycles: u16, mmu: &mut Mmu) {
        if !self.lcd_enabled(mmu) {
            self.reset(mmu);
            return;
        }

        self.dot_counter += cycles;

        match self.mode {
            PpuMode::OamScan if self.dot_counter >= 80 => {
                self.dot_counter -= 80;
                self.set_mode(PpuMode::Drawing, mmu);
            }

            PpuMode::Drawing if self.dot_counter >= 172 => {
                self.dot_counter -= 172;
                self.render_scanline(mmu);
                self.render_sprites_scanline(mmu);
                self.set_mode(PpuMode::HBlank, mmu);
            }

            PpuMode::HBlank if self.dot_counter >= 204 => {
                self.dot_counter -= 204;
                self.next_scanline(mmu);
            }

            PpuMode::VBlank if self.dot_counter >= 456 => {
                self.dot_counter -= 456;
                self.next_scanline(mmu);
            }

            _ => {}
        }
    }

    fn lcd_enabled(& self, mmu: &Mmu) -> bool {
        mmu.read8(0xFF40) & 0x80 != 0
    }

    fn reset(&mut self, mmu: &mut Mmu) {
        self.mode = PpuMode::HBlank;
        self.scanline = 0;
        self.dot_counter = 0;

        mmu.write_ly(0);

        let mut stat = mmu.read8(0xFF41);
        stat &= !0b11;
        mmu.write8(0xFF41, stat);
    }

    fn set_mode(&mut self, new_mode: PpuMode, mmu: &mut Mmu) {
        if self.mode == new_mode {
            return;
        }

        self.mode = new_mode;

        self.write_stat(mmu);
        self.check_stat_interrupt(mmu);
    }

    fn write_stat(&self, mmu: &mut Mmu) {
        let mut stat = mmu.read8(0xFF41);

        stat = (stat & !0b11) | self.mode_bits();

        mmu.write8(0xFF41, stat);
    }

    fn check_stat_interrupt(&self, mmu: &mut Mmu) {
        let stat = mmu.read8(0xFF41);

        let fire = match self.mode {
            PpuMode::HBlank  => stat & (1 << 3) != 0,
            PpuMode::VBlank  => stat & (1 << 4) != 0,
            PpuMode::OamScan => stat & (1 << 5) != 0,
            _ => false,
        };

        if fire {
            mmu.request_interrupt(Interrupt::LCDStat);
        }
    }

    fn mode_bits(&self) -> u8 {
        match self.mode {
            PpuMode::HBlank  => 0b00,
            PpuMode::VBlank  => 0b01,
            PpuMode::OamScan => 0b10,
            PpuMode::Drawing => 0b11,
        }
    }

    fn render_scanline(&mut self, mmu: &Mmu) {
        let ly = self.scanline;
        if ly >= 144 { return; }

        let map_base = 0x9800;
        let line_in_tile = ly % 8;

        for x in 0..160 {
            let tile_x = x / 8;
            let tile_index_addr = map_base + (tile_x as u16) + (ly / 8) as u16 * 32;
            let tile_index = mmu.read8(tile_index_addr as u16);
            let pixels = self.read_tile_line(mmu, tile_index, line_in_tile);

            let pixel_x = x % 8;
            self.framebuffer.pixels[ly as usize][x as usize] =
                Self::color_index_to_color(pixels[pixel_x as usize]);

        }
    }

    fn read_tile_line(&self, mmu: &Mmu, tile_index: u8, line: u8) -> [u8; 8] {
        let base = 0x8000;
        let tile_addr = base + (tile_index as u16) * 16;
        let byte1 = mmu.read8(tile_addr + line as u16 * 2);
        let byte2 = mmu.read8(tile_addr + line as u16 * 2 + 1);

        let mut pixels = [0u8; 8];
        for x in 0..8 {
            let bit = 7 - x;
            let color = ((byte2 >> bit) & 1) << 1 | ((byte1 >> bit) & 1);
            pixels[x as usize] = color;
        }
        pixels
    }

    fn render_sprites_scanline(&mut self, mmu: &Mmu) {
        let ly = self.scanline;
        let sprite_height = if mmu.read8(0xFF40) & 0x04 != 0 { 16 } else { 8 };
        let mut count = 0;

        for i in 0..40 {
            let base = 0xFE00 + i * 4;
            let y = mmu.read8(base) as i16 - 16;
            let x = mmu.read8(base + 1) as i16 - 8;
            let tile = mmu.read8(base + 2);
            let flags = mmu.read8(base + 3);

            if ly < y as u8 || ly >= (y + sprite_height) as u8 {
                continue;
            }

            count += 1;
            if count > 10 { break; }

            let mut line_in_tile = ly as i16 - y;
            if flags & 0x40 != 0 {
                line_in_tile = sprite_height - 1 - line_in_tile;
            }

            let pixels = self.read_tile_line(mmu, tile, line_in_tile as u8);

            for px in 0..8 {
                let mut pixel_x = px;
                if flags & 0x20 != 0 {
                    pixel_x = 7 - px;
                }
                let framebuffer_x = x + px;
                if framebuffer_x < 0 || framebuffer_x >= 160 {
                    continue;
                }

                let color = pixels[pixel_x as usize];
                if color == 0 { continue; }

                if flags & 0x80 != 0 {
                    let bg_color = self.framebuffer.pixels[ly as usize][framebuffer_x as usize];
                    if bg_color != 0 { continue; }
                }

                self.framebuffer.pixels[ly as usize][framebuffer_x as usize] =
                    Self::color_index_to_color(color);

            }
        }
    }

    
    fn next_scanline(&mut self, mmu: &mut Mmu) {
        self.scanline += 1;
        mmu.write_ly(self.scanline);

        self.check_lyc(mmu);

        if self.scanline == 144 {
            self.set_mode(PpuMode::VBlank, mmu);
            mmu.request_interrupt(Interrupt::VBlank);
            self.frame_ready = true;
        } else if self.scanline > 153 {
            self.scanline = 0;
            mmu.write_ly(0);
            self.set_mode(PpuMode::OamScan, mmu);
        } else {
            self.set_mode(PpuMode::OamScan, mmu);
        }
    }

    fn check_lyc(&self, mmu: &mut Mmu) {
        let lyc = mmu.read8(0xFF45);
        let mut stat = mmu.read8(0xFF41);

        if self.scanline == lyc {
            stat |= 1 << 2;
            if stat & (1 << 6) != 0 {
                mmu.request_interrupt(Interrupt::LCDStat);
            }
        } else {
            stat &= !(1 << 2);
        }

        mmu.write8(0xFF41, stat);
    }

    pub fn finalize_frame(&self, mmu: &Mmu, output: &mut [[u8;3]; 160*144]) {
        let bg_palette = mmu.read8(0xFF47);

        for y in 0..144 {
            for x in 0..160 {
                let color_index = self.framebuffer.pixels[y][x];
                let mapped = Self::apply_palette(color_index as u8, bg_palette);
                output[y*160 + x] = Self::color_to_rgb(mapped);
            }
        }
    }

    fn apply_palette(color_index: u8, palette: u8) -> u8 {
        match color_index {
            0 => (palette & 0b11),           // Bits 0-1
            1 => (palette >> 2) & 0b11,      // Bits 2-3
            2 => (palette >> 4) & 0b11,      // Bits 4-5
            3 => (palette >> 6) & 0b11,      // Bits 6-7
            _ => 0,
        }
    }

    fn color_to_rgb(color: u8) -> [u8;3] {
        match color {
            0 => [255, 255, 255], // Weiß
            1 => [192, 192, 192], // Hellgrau
            2 => [96, 96, 96],    // Dunkelgrau
            3 => [0, 0, 0],       // Schwarz
            _ => [255,255,255],
        }
    }

    fn color_index_to_color(color: u8) -> u32 {
        match color {
            0 => 0xFFFFFFFF,
            1 => 0xFFAAAAAA,
            2 => 0xFF555555,
            3 => 0xFF000000,
            _ => 0xFFFFFFFF,
        }
    }


}