pub mod noise_channel;
pub mod square_channel_1;
pub mod square_channel_2;
pub mod wave_channel;

use noise_channel::NoiseChannel;
use square_channel_1::SquareChannel1;
use square_channel_2::SquareChannel2;
use wave_channel::WaveChannel;

pub const SAMPLE_RATE: u32 = 44100;
pub const CPU_FREQ: u32 = 4_194_304;
pub const SAMPLE_PERIOD: u32 = CPU_FREQ / SAMPLE_RATE;

pub const DUTY_TABLE: [[u8; 8]; 4] = [
    [0,0,0,0,0,0,0,1],
    [1,0,0,0,0,0,0,1],
    [1,0,0,0,0,1,1,1],
    [0,1,1,1,1,1,1,0],
];

pub struct Apu {
    pub enabled: bool,

    cycles: u32,

    frame_step: u8,
    frame_counter: u32,

    pub ch1: SquareChannel1,
    pub ch2: SquareChannel2,
    pub ch3: WaveChannel,
    pub ch4: NoiseChannel,

    left_volume: u8,
    right_volume: u8,

    ch1_left: bool,
    ch1_right: bool,

    ch2_left: bool,
    ch2_right: bool,

    ch3_left: bool,
    ch3_right: bool,

    ch4_left: bool,
    ch4_right: bool,

    sample_timer: u32,
    pub sample_buffer_l: Vec<f32>,
    pub sample_buffer_r: Vec<f32>,
}

impl Apu {
    pub fn new() -> Self {
        Self {
            enabled: true,

            cycles: 0,

            frame_step: 0,
            frame_counter: 0,

            ch1: SquareChannel1::default(),
            ch2: SquareChannel2::default(),
            ch3: WaveChannel::default(),
            ch4: NoiseChannel::default(),

            left_volume: 7,
            right_volume: 7,

            ch1_left: true,
            ch1_right: true,
            ch2_left: true,
            ch2_right: true,
            ch3_left: true,
            ch3_right: true,
            ch4_left: true,
            ch4_right: true,

            sample_timer: 0,
            sample_buffer_l: Vec::new(),
            sample_buffer_r: Vec::new(),
        }
    }

    pub fn tick(&mut self, cycles: u32) {
        for _ in 0..cycles {
            self.tick_cycle();
        }
    }

    pub fn write_nr50(&mut self, value: u8) {
        self.left_volume = (value >> 4) & 0x07;
        self.right_volume = value & 0x07;
    }

    pub fn write_nr51(&mut self, value: u8) {
        self.ch1_left  = value & 0x10 != 0;
        self.ch1_right = value & 0x01 != 0;

        self.ch2_left  = value & 0x20 != 0;
        self.ch2_right = value & 0x02 != 0;

        self.ch3_left  = value & 0x40 != 0;
        self.ch3_right = value & 0x04 != 0;

        self.ch4_left  = value & 0x80 != 0;
        self.ch4_right = value & 0x08 != 0;
    }

    pub fn write_nr52(&mut self, value: u8) {
        self.enabled = value & 0x80 != 0;

        if !self.enabled {
            self.reset();
        }
    }

    pub fn read_nr52(&self) -> u8 {
        (if self.enabled     { 0x80 } else { 0 }) |
        (if self.ch4.enabled { 0x08 } else { 0 }) |
        (if self.ch3.enabled { 0x04 } else { 0 }) |
        (if self.ch2.enabled { 0x02 } else { 0 }) |
        (if self.ch1.enabled { 0x01 } else { 0 })
    }


    fn tick_cycle(&mut self) {
        if !self.enabled {
            return;
        }

        self.cycles += 1;

        self.frame_counter += 1;
        if self.frame_counter >= 8192 {
            self.frame_counter = 0;
            self.frame_step = (self.frame_step + 1) & 7;
            
            self.clock_frame_sequencer();
        }

        self.ch1.tick();
        self.ch2.tick();
        self.ch3.tick();
        self.ch4.tick();

        self.sample_timer += 1;
        if self.sample_timer >= SAMPLE_PERIOD {
            self.sample_timer -= SAMPLE_PERIOD;
        
            let (l, r) = self.mix_stereo();
            self.sample_buffer_l.push(l.clamp(-1.0, 1.0));
            self.sample_buffer_r.push(r.clamp(-1.0, 1.0));
        }
    }

    fn reset(&mut self) {
        self.ch1.enabled = false;
        self.ch2.enabled = false;
        self.ch3.enabled = false;
        self.ch4.enabled = false;
    }

    fn clock_frame_sequencer(&mut self) {
        match self.frame_step {
            2 => {
                self.ch1.clock_length();
                self.ch1.clock_sweep();
                self.ch2.clock_length();
                self.ch3.clock_length();
                self.ch4.clock_length();
            }
            0 | 4 | 6 => {
                self.ch1.clock_length();
                self.ch2.clock_length();
                self.ch3.clock_length();
                self.ch4.clock_length();
            }
            7 => {
                self.ch1.clock_envelope();
                self.ch2.clock_envelope();
                self.ch4.clock_envelope();
            }
            _ => {}
        }
    }

    fn mix_stereo(&self) -> (f32, f32) {
        if !self.enabled {
            return (0.0, 0.0);
        }

        let mut left = 0.0;
        let mut right = 0.0;

        let ch1 = self.ch1.sample();
        let ch2 = self.ch2.sample();
        let ch3 = self.ch3.sample();
        let ch4 = self.ch4.sample();

        if self.ch1_left  { left  += ch1; }
        if self.ch1_right { right += ch1; }

        if self.ch2_left  { left  += ch2; }
        if self.ch2_right { right += ch2; }

        if self.ch3_left  { left  += ch3; }
        if self.ch3_right { right += ch3; }

        if self.ch4_left  { left  += ch4; }
        if self.ch4_right { right += ch4; }

        let l_vol = self.left_volume as f32 / 7.0;
        let r_vol = self.right_volume as f32 / 7.0;

        (left * l_vol, right * r_vol)
    }

}
