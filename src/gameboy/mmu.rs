use super::{cpu::Interrupt, super::gameboy::{timer::{Timer, TimerAddr}, joypad::{Joypad, Key}}};


/// struct that represent the Memory Managment Unit (MMU)
pub struct Mmu {
    rom: Vec<u8>,
    vram: [u8; 0x2000],
    eram: Vec<u8>,
    wram: [u8; 0x2000],
    oam:  [u8; 0xA0],
    hram: [u8; 0x7F],
    io:   [u8; 0x80],
    ie:   u8,

    joypad: Joypad,
    
    cartridge_type: u8,

    rom_bank: u8, 
    ram_bank: u8,
    ram_enabled: bool,

    banking_mode: u8,

    timer: Timer,

    mbc3_rtc_sel: Option<u8>,
}

impl Mmu {
    pub fn new(rom: Vec<u8>, timer: Timer) -> Self {
        let cartridge_type = rom.get(0x0147).copied().unwrap_or(0);
        let ram_size = Self::ram_size_from_header(
            rom.get(0x0149).copied().unwrap_or(0)
        );
        
        Self {
            rom,
            vram: [0; 0x2000],
            eram: vec![0; ram_size],
            wram: [0; 0x2000],
            oam: [0; 0xA0],
            hram: [0; 0x7F],
            io: [0xFF; 0x80],
            ie: 0,  

            joypad: Joypad::new(),

            cartridge_type,
            
            rom_bank: 1,
            ram_bank: 0,
            ram_enabled: false,

            banking_mode: 0,

            timer,

            mbc3_rtc_sel: None,
        }
    }

    pub fn read8(&self, addr: u16) -> u8 {      
        match addr {
            0x0000..=0x3FFF => self.rom.get(addr as usize).copied().unwrap_or(0xFF), 
            0x4000..=0x7FFF => {
                let bank_addr = (self.rom_bank as usize) * 0x4000 + ((addr - 0x4000) as usize);
                self.rom.get(bank_addr).copied().unwrap_or(0xFF)
            },
            0x8000..=0x9FFF => self.vram[(addr - 0x8000) as usize],
            0xA000..=0xBFFF => {
                if self.mbc3_rtc_sel.is_some() { return 0xFF; }

                if !self.ram_enabled || self.eram.is_empty() { return 0xFF; }

                let bank = self.ram_bank as usize;
                let offset = bank * 0x2000 + (addr - 0xA000) as usize;
                
                self.eram.get(offset).copied().unwrap_or(0xFF)
            },
            0xC000..=0xDFFF => self.wram[(addr - 0xC000) as usize],
            0xE000..=0xFDFF => self.wram[(addr - 0xE000) as usize % 0x2000],
            0xFE00..=0xFE9F => self.oam[(addr - 0xFE00) as usize],
            0xFF00          => self.joypad.read(),
            0xFF0F          => self.io[0x0F] | 0xE0,
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize],
            0xFF00..=0xFF7F => self.io[(addr - 0xFF00) as usize],
            0xFFFF          => self.ie,
            _               => 0xFF,
        }
    }

    pub fn write8(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram_enabled = (value & 0x0F) == 0x0A,
            0x2000..=0x3FFF => {
                if self.cartridge_type == 0x00 { return; }

                if self.is_mbc3() {
                    let mut bank = value & 0x7F;
                    if bank == 0 { bank = 1; }
                    self.rom_bank = bank;
                    return;
                }

                if self.is_mbc1() {
                    let mut bank = value & 0x1F;
                    if bank == 0 { bank = 1; }
                    self.rom_bank = (self.rom_bank & 0xE0) | bank;
                }
            },
            0x4000..=0x5FFF => {
                if self.cartridge_type == 0x00 { return; }

                if self.is_mbc3() {
                    if (0x08..=0x0C).contains(&value) {
                        self.mbc3_rtc_sel = Some(value);
                    } else {
                        self.mbc3_rtc_sel = None;
                        self.ram_bank = value & 0x03;
                    }
                    return;
                }

                if self.is_mbc1() {
                    let bank_bits = value & 0x03;
                    
                    if self.banking_mode == 0 {
                        self.rom_bank = (self.rom_bank & 0x1F) | (bank_bits << 5);
                    } else {
                        self.ram_bank = bank_bits;
                    }
                }
            },
            0x6000..=0x7FFF => {
                if self.cartridge_type == 0x00 { return; }

                if self.is_mbc1() {
                    self.banking_mode = value & 0x01;
                }
            },
            0x8000..=0x9FFF => self.vram[(addr - 0x8000) as usize] = value,
            0xA000..=0xBFFF => {
                if self.mbc3_rtc_sel.is_some() { return; }

                if !self.ram_enabled || self.eram.is_empty() { return; }

                let bank = self.ram_bank as usize;
                let offset = bank * 0x2000 + (addr - 0xA000) as usize;

                if let Some(byte) = self.eram.get_mut(offset) {
                    *byte = value;
                }
            },
            0xC000..=0xDFFF => self.wram[(addr - 0xC000) as usize] = value,
            0xE000..=0xFDFF => self.wram[(addr - 0xE000) as usize] = value,
            0xFE00..=0xFE9F => self.oam[(addr - 0xFE00) as usize] = value,
            0xFF00          => self.joypad.write(value),
            0xFF02          => {
                self.io[0x02] = value;
            
                if value & 0x80 != 0 {
                    self.io[0x01] = 0xFF;
                    self.io[0x02] = value & 0x7F; 
                    self.request_interrupt(Interrupt::Serial);
                }
            },
            0xFF04          => {
                self.timer.reset_divider();
                self.io[0x04] = 0
            }, 
            0xFF0F          => self.io[0x0F] = (self.io[0x0F] & 0xE0) | (value & 0x1F),
            0xFF41          => {
                let read_only = self.io[0x41] & 0b0000_0111;
                let writeable = value & 0b0111_1000;
                self.io[0x41] = read_only | writeable | 0x80;
            },         
            0xFF44          => {},
            0xFF46          => {
                self.io[0x46] = value;

                let source = (value as u16) << 8;        
                for i in 0..0xA0 {
                    let byte = self.read8(source + i);
                    self.oam[i as usize] = byte;
                }
            },
            0xFF47..=0xFF49 => self.io[(addr - 0xFF00) as usize] = value,
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize] = value,
            0xFF00..=0xFF7F => self.io[(addr - 0xFF00) as usize] = value,
            0xFFFF          => self.ie = value & 0x1F,
            _               => {},
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

    pub fn tick(&mut self, cycles: u8) {
        let tac = self.read8(TimerAddr::TAC as u16);
        let tima = self.read8(TimerAddr::TIMA as u16);
        let tma = self.read8(TimerAddr::TMA as u16);
        
        let update = self.timer.update(cycles, tac, tima, tma);
        
        if let Some(div) = update.new_div {
            self.write_div(div);
        }
        if let Some(tima) = update.new_tima {
            self.write8(TimerAddr::TIMA as u16, tima);
        }
        if update.timer_interrupt {
            self.request_interrupt(Interrupt::Timer);
        }
    }

    pub fn request_interrupt(&mut self, interrupt: Interrupt) {
        self.io[0x0F] |= 1 << (interrupt as u8);
    }

    pub fn write_ly(&mut self, value: u8) {
        self.io[0x44] = value;
    }

    pub fn write_div(&mut self, value: u8) {
        self.io[0x04] = value;
    }

    fn ram_size_from_header(value: u8) -> usize {
        match value {
            0x00 => 0,
            0x01 => 2 * 1024,
            0x02 => 8 * 1024,
            0x03 => 32 * 1024,
            0x04 => 128 * 1024,
            0x05 => 64 * 1024,
            _ => 0,
        }
    }

    pub fn key_down(&mut self, key: Key) {
        if self.joypad.press(key) {
            self.request_interrupt(Interrupt::Joypad);
        }
    }

    pub fn key_up(&mut self, key: Key) {
        self.joypad.release(key);
    }

    fn is_mbc1(&self) -> bool {
        matches!(self.cartridge_type, 0x01 | 0x02 | 0x03)
    }

    fn is_mbc3(&self) -> bool {
        matches!(self.cartridge_type, 0x0F | 0x10 | 0x11 | 0x12 | 0x13)
    }
}