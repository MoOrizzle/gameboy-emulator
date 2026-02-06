use super::super::apu::DUTY_TABLE;


#[derive(Default)]
pub struct SquareChannel1 {
    pub enabled: bool,
    length_enable: bool,

    dac_enabled: bool,

    pub duty: u8,
    duty_step: u8,

    pub frequency: u16,
    timer: i16,

    pub length_counter: u8,

    // Envelope
    volume: u8,
    envelope_period: u8,
    envelope_timer: u8,
    envelope_increase: bool,

    // Sweep
    sweep_period: u8,
    sweep_timer: u8,
    sweep_shift: u8,
    sweep_decrease: bool,
    sweep_shadow_freq: u16,
}

impl SquareChannel1 {
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

    pub fn write_envelope(&mut self, value: u8) {
        self.update_dac(value);

        self.volume = (value >> 4) & 0x0F;
        self.envelope_increase = (value & 0x08) != 0;
        self.envelope_period = value & 0x07;
    }

    pub fn write_sweep(&mut self, value: u8) {
        self.sweep_period = (value >> 4) & 0x07;
        self.sweep_decrease = (value & 0x08) != 0;
        self.sweep_shift = value & 0x07;
    }

    pub fn write_freq_high(&mut self, value: u8) {
        self.frequency = (self.frequency & 0x00FF) | (((value & 0x07) as u16) << 8);
        self.length_enable = value & 0x40 != 0;
        
        if value & 0x80 != 0 {
            self.trigger();
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

        // Envelope
        self.envelope_timer = self.envelope_period;

        // Sweep
        self.sweep_shadow_freq = self.frequency;
        self.sweep_timer = if self.sweep_period == 0 { 8 } else { self.sweep_period };

        if self.sweep_shift > 0 {
            self.calculate_sweep();
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

    pub fn clock_sweep(&mut self) {
        if self.sweep_period == 0 {
            return;
        }

        if self.sweep_timer > 0 {
            self.sweep_timer -= 1;
        }

        if self.sweep_timer == 0 {
            self.sweep_timer = if self.sweep_period == 0 { 8 } else { self.sweep_period };

            if self.sweep_shift > 0 {
                let new_freq = self.calculate_sweep();

                if new_freq <= 2047 {
                    self.frequency = new_freq;
                    self.sweep_shadow_freq = new_freq;

                    self.calculate_sweep();
                }
            }
        }
    }

    fn calculate_sweep(&mut self) -> u16 {
        let delta = self.sweep_shadow_freq >> self.sweep_shift;
    
        let new_freq = if self.sweep_decrease {
            self.sweep_shadow_freq.wrapping_sub(delta)
        } else {
            self.sweep_shadow_freq.wrapping_add(delta)
        };
    
        if new_freq > 2047 {
            self.enabled = false;
        }
    
        new_freq
    }

}