#[derive(Default)]
pub struct NoiseChannel {
    pub enabled: bool,
    length_enable: bool,

    length_counter: u8,

    // Envelope
    volume: u8,
    envelope_period: u8,
    envelope_timer: u8,
    envelope_increase: bool,

    // Noise
    lfsr: u16,
    clock_shift: u8,
    divisor_code: u8,
    width_mode_7: bool,

    timer: i16,
}

impl NoiseChannel {
    pub fn write_nr41(&mut self, value: u8) {
        self.length_counter = 64 - (value & 0x3F);
    }

    pub fn write_nr42(&mut self, value: u8) {
        self.volume = (value >> 4) & 0x0F;
        self.envelope_increase = (value & 0x08) != 0;
        self.envelope_period = value & 0x07;
    }

    pub fn write_nr43(&mut self, value: u8) {
        self.clock_shift = value >> 4;
        self.width_mode_7 = value & 0x08 != 0;
        self.divisor_code = value & 0x07;
    }

    pub fn write_nr44(&mut self, value: u8) {
        self.length_enable = value & 0x40 != 0;
    }

    fn reload_timer(&self) -> i16 {
        const DIVISORS: [i16; 8] = [8,16,32,48,64,80,96,112];
        DIVISORS[self.divisor_code as usize] << self.clock_shift
    }

    pub fn trigger(&mut self) {
        self.enabled = true;

        if self.length_counter == 0 {
            self.length_counter = 64;
        }

        self.envelope_timer = self.envelope_period;

        self.lfsr = 0x7FFF; // alle Bits = 1
        self.timer = self.reload_timer();
    }

    pub fn tick(&mut self) {
        if !self.enabled {
            return;
        }

        self.timer -= 1;
        if self.timer <= 0 {
            self.timer = self.reload_timer();

            let bit0 = self.lfsr & 1;
            let bit1 = (self.lfsr >> 1) & 1;
            let xor = bit0 ^ bit1;

            self.lfsr >>= 1;
            self.lfsr |= xor << 14;

            if self.width_mode_7 {
                self.lfsr &= !(1 << 6);
                self.lfsr |= xor << 6;
            }
        }
    }

    pub fn clock_envelope(&mut self) {
        if self.envelope_period == 0 {
            return;
        }

        if self.envelope_timer > 0 {
            self.envelope_timer -= 1;
        }

        if self.envelope_timer == 0 {
            self.envelope_timer = self.envelope_period;

            if self.envelope_increase && self.volume < 15 {
                self.volume += 1;
            } else if !self.envelope_increase && self.volume > 0 {
                self.volume -= 1;
            }
        }
    }

    pub fn clock_length(&mut self) {
        if !self.length_enable { return; }

        if self.length_counter > 0 {
            self.length_counter -= 1;
            if self.length_counter == 0 {
                self.enabled = false;
            }
        }
    }

    pub fn sample(&self) -> f32 {
        if !self.enabled || self.volume == 0 {
            return 0.0;
        }

        let bit = !(self.lfsr & 1) as f32;
        let amp = self.volume as f32 / 15.0;

        (bit * 2.0 - 1.0) * amp
    }

}