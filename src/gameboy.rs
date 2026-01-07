pub mod cpu;
pub mod mmu;

use cpu::Cpu;
use mmu::Mmu;

pub struct GameBoy {
    cpu: Cpu,
    mmu: Mmu,
}

impl GameBoy {
    pub fn new(rom: Vec<u8>) -> Self {
        Self { cpu: Cpu::new(), mmu: Mmu::new(rom) }
    }

    pub fn step(&mut self) {
        let _cycles = self.cpu.step(&mut self.mmu);
    }
}