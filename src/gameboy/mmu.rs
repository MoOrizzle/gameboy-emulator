/// struct that represent the Memory Managment Unit (MMU)
pub struct Mmu {
    rom: Vec<u8>,
    wram: [u8; 0x2000],
    hram: [u8; 0x7F],
}

impl Mmu {
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            rom,
            wram: [0; 0x2000],
            hram: [0; 0x7F],
        }
    }

    pub fn read8(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7FFF => self.rom[addr as usize],
            0xC000..=0xDFFF => self.wram[(addr - 0xC000) as usize],
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize],
            _ => 0xFF,
        }
    }

    pub fn write8(&mut self, addr: u16, value: u8) {
        match addr {
            0xC000..=0xDFFF => self.wram[(addr - 0xC000) as usize] = value,
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize] = value,
            _ => {}
        }
    }

    pub fn read16(&self, addr: u16) -> u16 {
        let lower_byte = self.read8(addr) as u16;
        let higher_byte = self.read8(addr + 1) as u16;

        (higher_byte << 8) | lower_byte
    }

    pub fn write16(&mut self, addr: u16, value: u16) {
        let lower_byte = value & 0x00FF;
        let higher_byte = value >> 8;

        self.write8(addr, lower_byte as u8);
        self.write8(addr + 1, higher_byte as u8);
    }
}