pub mod cpu;
pub mod screen;
pub mod joypad;
pub mod mmu;
pub mod ppu;

use cpu::Cpu;
use joypad::{Joypad, Key};
use mmu::Mmu;
use ppu::Ppu;


pub struct GameBoy {
    cpu: Cpu,
    joypad: Joypad,
    mmu: Mmu,
    ppu: Ppu,
}

impl GameBoy {
    pub fn new(rom: Vec<u8>) -> Self {
        Self { 
            cpu: Cpu::new(), 
            joypad: Joypad::new(),
            mmu: Mmu::new(rom),
            ppu: Ppu::new(),
        }
    }

    pub fn step(&mut self) {
        let cycles = self.cpu.step(&mut self.mmu);

        //self.timer.update(&mut self.mmu, cycles);
        self.ppu.step(cycles as u16, &mut self.mmu);

        self.cpu.handle_interrupts(&mut self.mmu);


        if self.ppu.frame_ready {
            //self.ppu.finalize_frame(&self.mmu, &mut display_buffer);
            //renderer.render(&display_buffer);
            self.ppu.frame_ready = false;
        }
    }



    pub fn key_down(&mut self, key: Key) {
        self.joypad.press(key, &mut self.mmu);
    }

    pub fn key_up(&mut self, key: Key) {
        self.joypad.release(key);
    }
}