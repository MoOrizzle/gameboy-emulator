pub mod cpu;
pub mod joypad;
pub mod mmu;
pub mod ppu;
pub mod screen;
pub mod timer;

use cpu::Cpu;
use joypad::Key;
use mmu::Mmu;
use ppu::Ppu;
use timer::Timer;


pub struct GameBoy {
    cpu: Cpu,
    mmu: Mmu,
    pub ppu: Ppu,
}

impl GameBoy {
    pub fn new(rom: Vec<u8>) -> Self {
        Self { 
            cpu: Cpu::new(), 
            mmu: Mmu::new(rom, Timer::new()),
            ppu: Ppu::new(),
        }
    }

    pub fn step(&mut self) -> bool {
        let cycles = self.cpu.step(&mut self.mmu);
        
        self.mmu.tick(cycles);
        self.ppu.step(cycles as u16, &mut self.mmu);

        if self.ppu.frame_ready {
            self.ppu.frame_ready = false;
            return true;
        }

        false
    }

    pub fn key_down(&mut self, key: Key) {
        self.mmu.key_down(key);
    }

    pub fn key_up(&mut self, key: Key) {
        self.mmu.key_up(key);
    }
}