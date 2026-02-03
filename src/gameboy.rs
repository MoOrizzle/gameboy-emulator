pub mod cpu;
pub mod joypad;
pub mod mmu;
pub mod timer;

use cpu::Cpu;
use joypad::Key;
use mmu::Mmu;
use timer::Timer;


pub struct GameBoy {
    cpu: Cpu,
    mmu: Mmu,
}

impl GameBoy {
    pub fn new(rom: Vec<u8>) -> Self {
        Self { 
            cpu: Cpu::new(), 
            mmu: Mmu::new(rom, Timer::new()),
        }
    }

    pub fn step(&mut self) -> bool {
        let cycles = self.cpu.step(&mut self.mmu);
        
        self.mmu.tick(cycles);

        false
    }

    pub fn key_down(&mut self, key: Key) {
        self.mmu.key_down(key);
    }

    pub fn key_up(&mut self, key: Key) {
        self.mmu.key_up(key);
    }
}