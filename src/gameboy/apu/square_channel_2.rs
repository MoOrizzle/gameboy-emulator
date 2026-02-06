use super::super::apu::DUTY_TABLE;


#[derive(Default)]
pub struct SquareChannel2 {
    pub enabled: bool,
    length_enable: bool,

    dac_enabled: bool,

    duty: u8,
    duty_step: u8,

    frequency: u16,
    timer: i16,

    length_counter: u8,

    // Envelope
    volume: u8,
    envelope_period: u8,
    envelope_timer: u8,
    envelope_increase: bool,
}

impl SquareChannel2 {
    fn update_dac(&mut self, value: u8) {
        self.dac_enabled = (value & 0xF8) != 0;
    }

    pub fn tick(&mut self) {
        if !self.enabled {
            return;
        }

        self.timer -= 1;
        if self.timer <= 0 {
            self.timer = (2048 - self.frequency) as i16 * 4;
            self.duty_step = (self.duty_step + 1) & 7;
        }
    }

    pub fn sample(&self) -> f32 {
        if !self.enabled || !self.dac_enabled {
            return 0.0;
        }

        let amp = self.volume as f32 / 15.0;

        if DUTY_TABLE[self.duty as usize][self.duty_step as usize] == 1 { amp } else { -amp }
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

    pub fn trigger(&mut self) {
        if !self.dac_enabled { return; }

        self.enabled = true;

        self.timer = (2048 - self.frequency) as i16 * 4;
        self.duty_step = 0;

        if self.length_counter == 0 {
            self.length_counter = 64;
        }

        self.envelope_timer = self.envelope_period;
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

            if self.envelope_increase {
                if self.volume < 15 {
                    self.volume += 1;
                }
            } else {
                if self.volume > 0 {
                    self.volume -= 1;
                }
            }
        }
    }

    pub fn write_duty_length(&mut self, value: u8) {
        self.duty = value >> 6;
        self.length_counter = 64 - (value & 0x3F);
    }

    pub fn write_envelope(&mut self, value: u8) {
        self.update_dac(value);

        self.volume = (value >> 4) & 0x0F;
        self.envelope_increase = (value & 0x08) != 0;
        self.envelope_period = value & 0x07;
    }

    pub fn write_freq_low(&mut self, value: u8) {
        self.frequency = (self.frequency & 0x0700) | value as u16;
    }

    pub fn write_freq_high(&mut self, value: u8) {
        self.frequency = (self.frequency & 0x00FF) | (((value & 0x07) as u16) << 8);
        self.length_enable = value & 0x40 != 0;

        if value & 0x80 != 0 {
            self.trigger();
        }
    }

}