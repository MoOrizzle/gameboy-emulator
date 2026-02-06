use super::{cpu::Interrupt, screen::framebuffer::Framebuffer, mmu::Mmu};

#[derive(PartialEq, Clone, Copy)]
pub enum PpuMode {
    HBlank,
    VBlank,
    OamScan,
    Drawing,
}

pub struct Ppu {
    pub frame_ready: bool,
    pub framebuffer: Framebuffer,
    mode: PpuMode,
    scanline: u8,
    dot_counter: u16,
    bg_color_index: [u8; 160],
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            framebuffer: Framebuffer::new(),
            mode: PpuMode::OamScan,
            scanline: 0,
            frame_ready: false,
            dot_counter: 0,
            bg_color_index: [0; 160],
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
                self.enter_mode(PpuMode::Drawing, mmu);
            }
            PpuMode::Drawing if self.dot_counter >= 172 => {
                self.dot_counter -= 172;
                self.render_scanline(mmu);
                self.render_sprites_scanline(mmu);
                self.enter_mode(PpuMode::HBlank, mmu);
            }
            PpuMode::HBlank if self.dot_counter >= 204 => {
                self.dot_counter -= 204;
                self.advance_scanline(mmu);
            }
            PpuMode::VBlank if self.dot_counter >= 456 => {
                self.dot_counter -= 456;
                self.advance_scanline(mmu);
            }
            _ => {}
        }
    }

    fn lcd_enabled(&self, mmu: &Mmu) -> bool {
        mmu.read8(0xFF40) & 0x80 != 0
    }

    fn reset(&mut self, mmu: &mut Mmu) {
        self.dot_counter = 0;
        self.mode = PpuMode::OamScan;
        self.scanline = 0;
        mmu.write_ly(0);
    }

    fn enter_mode(&mut self, new_mode: PpuMode, mmu: &mut Mmu) {
        if self.mode == new_mode {
            return;
        }

        self.mode = new_mode;
        self.write_stat(mmu);

        let stat = mmu.read8(0xFF41);
        let fire = match new_mode {
            PpuMode::HBlank  => stat & (1 << 3) != 0,
            PpuMode::VBlank  => stat & (1 << 4) != 0,
            PpuMode::OamScan => stat & (1 << 5) != 0,
            _ => false,
        };

        if fire {
            mmu.request_interrupt(Interrupt::LCDStat);
        }
    }

    fn write_stat(&self, mmu: &mut Mmu) {
        let mut stat = mmu.read8(0xFF41);

        let bits = self.mode as u8;
        stat = (stat & !0b11) | bits;

        mmu.write8(0xFF41, stat);
    }

    fn render_scanline(&mut self, mmu: &Mmu) {
        let ly = self.scanline as u16;
        if ly >= 144 { return; }

        let lcdc = mmu.read8(0xFF40);
        let bg_enabled = lcdc & 0x01 != 0;
        let window_enabled = lcdc & 0x20 != 0;

        if !bg_enabled && !window_enabled { return; }

        let scx = mmu.read8(0xFF43) as u16;
        let scy = mmu.read8(0xFF42) as u16;
        let wy = mmu.read8(0xFF4A) as u16;
        let wx = mmu.read8(0xFF4B).wrapping_sub(7) as u16;

        let tile_data_area = lcdc & 0x10 != 0;
        let bg_map_base = if lcdc & 0x08 != 0 { 0x9C00 } else { 0x9800 };
        let win_map_base = if lcdc & 0x40 != 0 { 0x9C00 } else { 0x9800 };

        let bgp = mmu.read8(0xFF47);

        for screen_x in 0..160u16 {
            let use_window = window_enabled && ly >= wy && screen_x >= wx;

            let (map_base, x, y) = 
                if use_window {
                    (win_map_base, screen_x - wx, ly - wy)
                } else {
                    if !bg_enabled {
                        self.bg_color_index[screen_x as usize] = 0;
                        self.framebuffer.pixels[ly as usize][screen_x as usize] = Self::color_index_to_color(0);
                        continue;
                    }
                    (bg_map_base, (screen_x + scx) & 0xFF, (ly + scy) & 0xFF)
                };
            
            let tile_x = x / 8;
            let tile_y = y / 8;
            let pixel_x = x % 8;
            let pixel_y = y % 8;
            
            let tile_index_addr = map_base + tile_y * 32 + tile_x;
            let tile_index = mmu.read8(tile_index_addr);

            let tile_addr = if tile_data_area {
                0x8000 + (tile_index as u16) * 16
            } else {
                let base = 0x9000_u16;
                let signed_offset = (tile_index as i8) as i16 * 16;
                base.wrapping_add(signed_offset as u16)
            };

            let tile_addr = tile_addr + pixel_y * 2;

            let b1 = mmu.read8(tile_addr);
            let b2 = mmu.read8(tile_addr + 1);
            
            let bit = 7 - pixel_x;
            let color_index = ((b2 >> bit) & 1) << 1 | ((b1 >> bit) & 1);
            let color = Self::apply_palette(color_index, bgp);

            self.bg_color_index[screen_x as usize] = color_index;
            self.framebuffer.pixels[ly as usize][screen_x as usize] = Self::color_index_to_color(color);
        }
    }

    fn render_sprites_scanline(&mut self, mmu: &Mmu) {
        let ly = self.scanline as i16;

        if ly >= 144 { return; }

        let lcdc = mmu.read8(0xFF40);
        
        if lcdc & 0x02 == 0 { return; }

        let sprite_height = if lcdc & 0x04 != 0 { 16 } else { 8 };
        let mut count = 0;

        for i in 0..40 {
            let base = 0xFE00 + i * 4;
            let y = mmu.read8(base) as i16 - 16;
            let x = mmu.read8(base + 1) as i16 - 8;
            let mut tile = mmu.read8(base + 2);
            let flags = mmu.read8(base + 3);

            if ly < y || ly >= y + sprite_height { continue; }
            if x <= -8 || x >= 160 { continue; }

            count += 1;
            if count > 10 { break; }

            if sprite_height == 16 {
                tile &= 0xFE;
            }

            let mut line_in_tile = ly - y;

            if flags & 0x40 != 0 { 
                line_in_tile = sprite_height - 1 - line_in_tile; 
            }

            let actual_tile =
                if sprite_height == 16 && line_in_tile >= 8 
                { tile + 1 } else { tile };
            
            let tile_line = (line_in_tile % 8) as u8;

            let tile_addr = 0x8000 + actual_tile as u16 * 16 + tile_line as u16 * 2;
            let byte1 = mmu.read8(tile_addr);
            let byte2 = mmu.read8(tile_addr + 1);

            for px in 0..8 {
                let framebuffer_x = x + px;
                if framebuffer_x < 0 || framebuffer_x >= 160 { continue; }

                let mut pixel_x = px;

                if flags & 0x20 != 0 { 
                    pixel_x = 7 - px; 
                }
                
                let bit = 7 - pixel_x;
                let color_index = ((byte2 >> bit) & 1) << 1 | ((byte1 >> bit) & 1);
                if color_index == 0 { continue; }

                if flags & 0x80 != 0 {
                    if self.bg_color_index[framebuffer_x as usize] != 0 { continue; }
                }

                let obp = if flags & 0x10 != 0 { 
                    mmu.read8(0xFF49) 
                } else { 
                    mmu.read8(0xFF48) 
                };
                
                let color = Self::apply_palette(color_index, obp);
                self.framebuffer.pixels[ly as usize][framebuffer_x as usize] = Self::color_index_to_color(color);
            }
        }
    }

    fn advance_scanline(&mut self, mmu: &mut Mmu) {
        self.scanline += 1;

        if self.scanline > 153 {
            self.scanline = 0;
        }

        mmu.write_ly(self.scanline);
        self.check_lyc(mmu);

        if self.scanline == 144 {
            self.enter_mode(PpuMode::VBlank, mmu);
            mmu.request_interrupt(Interrupt::VBlank);
            self.frame_ready = true;
        }

        if self.scanline < 144 {
            self.enter_mode(PpuMode::OamScan, mmu);
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

    fn apply_palette(color_index: u8, palette: u8) -> u8 {
        match color_index {
            0 => palette & 0b11,       
            1 => (palette >> 2) & 0b11,
            2 => (palette >> 4) & 0b11,
            3 => (palette >> 6) & 0b11,
            _ => 0,
        }
    }

    //TODO: Make colors changeable through config?
    fn color_index_to_color(color: u8) -> u32 {
        match color {
            0 => 0xFFE0F8D0,  //0xFFFFFFFF
            1 => 0xFF88C070,  //0xFFAAAAAA
            2 => 0xFF346856,  //0xFF555555
            3 => 0xFF081820,  //0xFF000000
            _ => 0xFFFFFFFF,
        }
    }
}