pub mod audio;
pub mod apu;
pub mod cpu;
pub mod joypad;
pub mod mmu;
pub mod ppu;
pub mod screen;
pub mod timer;

use apu::Apu;
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
            mmu: Mmu::new(
                rom, 
                Apu::new(), 
                Timer::new()
            ),
            ppu: Ppu::new(),
        }
    }

    pub fn step(&mut self) -> bool {
        let cycles = self.cpu.step(&mut self.mmu);
        
        self.mmu.tick(cycles);
        self.mmu.tick_apu(cycles);
        self.ppu.step(cycles as u16, &mut self.mmu);

        if self.ppu.frame_ready {
            self.ppu.frame_ready = false;
            return true;
        }

        false
    }

    pub fn get_audio_samples(&mut self) -> (Vec<f32>, Vec<f32>) {
        self.mmu.get_audio_samples()
    }

    pub fn key_down(&mut self, key: Key) {
        self.mmu.key_down(key);
    }

    pub fn key_up(&mut self, key: Key) {
        self.mmu.key_up(key);
    }
}