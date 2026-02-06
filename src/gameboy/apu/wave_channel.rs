#[derive(Default)]
pub struct WaveChannel {
    pub enabled: bool,
    dac_enabled: bool,

    frequency: u16,
    timer: i16,

    length_counter: u16,

    volume_code: u8,

    wave_ram: [u8; 16],
    position: u8,
}

impl WaveChannel {
    pub fn write_nr30(&mut self, value: u8) {
        self.dac_enabled = value & 0x80 != 0;
        if self.dac_enabled { return; }
        self.enabled = false;
    }
    
    pub fn write_nr31(&mut self, value: u16) {
        self.length_counter = 256 - value;
    }

    pub fn write_nr32(&mut self, value: u8) {
        self.volume_code = (value >> 5) & 0x03;
    }

    pub fn write_freq_low(&mut self, value: u8) {
        self.frequency = (self.frequency & 0x0700) | value as u16;
    }

    pub fn write_freq_high(&mut self, value: u8) {
        self.frequency = (self.frequency & 0x00FF) | (((value & 0x07) as u16) << 8);

        if value & 0x80 != 0 {
            self.trigger();
        }
    }

    pub fn trigger(&mut self) {
        if !self.dac_enabled {
            return;
        }

        self.enabled = true;
        self.timer = (2048 - self.frequency) as i16 * 2;
        self.position = 0;

        if self.length_counter == 0 {
            self.length_counter = 256;
        }
    }

    pub fn tick(&mut self) {
        if !self.enabled {
            return;
        }

        self.timer -= 1;
        if self.timer <= 0 {
            self.timer = (2048 - self.frequency) as i16 * 2;
            self.position = (self.position + 1) & 31;
        }
    }

    pub fn sample(&self) -> f32 {
        if !self.enabled || self.volume_code == 0 {
            return 0.0;
        }

        let byte = self.wave_ram[(self.position / 2) as usize];
        let sample = if self.position & 1 == 0 {
            byte >> 4
        } else {
            byte & 0x0F
        };

        let mut s = sample as f32 / 15.0;

        s /= match self.volume_code {
            1 => 1.0,
            2 => 2.0,
            3 => 4.0,
            _ => 1.0,
        };

        (s * 2.0) - 1.0
    }

    pub fn clock_length(&mut self) {
        if self.length_counter > 0 {
            self.length_counter -= 1;
            if self.length_counter == 0 {
                self.enabled = false;
            }
        }
    }

    pub fn write_wave_ram(&mut self, addr: u16, value: u8) {
        self.wave_ram[(addr & 0x0F) as usize] = value;
    }
    
    pub fn read_wave_ram(&self, addr: u16) -> u8 {
        self.wave_ram[(addr & 0x0F) as usize]
    }

}