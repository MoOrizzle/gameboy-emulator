pub struct Timer {
    internal_counter: u16
}

pub struct TimerUpdate {
    pub new_div: Option<u8>,
    pub new_tima: Option<u8>,
    pub timer_interrupt: bool,
}

pub enum TimerAddr {
    DIV = 0xFF04,
    TIMA = 0xFF05,
    TMA = 0xFF06,
    TAC = 0xFF07
}

impl Timer {
    pub fn new() -> Self {
        Self {
            internal_counter: 0,
        }
    }

    pub fn update(&mut self, cycles: u8, tac: u8, tima: u8, tma: u8) -> TimerUpdate {
        let mut result = TimerUpdate {
            new_div: None,
            new_tima: None,
            timer_interrupt: false,
        };
        
        let timer_enabled = tac & 0x04 != 0;
        
        for _ in 0..cycles {
            let old_counter = self.internal_counter;
            self.internal_counter = self.internal_counter.wrapping_add(1);
            
            // DIV Update
            if (self.internal_counter >> 8) != (old_counter >> 8) {
                result.new_div = Some((self.internal_counter >> 8) as u8);
            }
            
            // TIMA Update
            if timer_enabled {
                let bit_position = match tac & 0x03 {
                    0 => 9,
                    1 => 3,
                    2 => 5,
                    3 => 7,
                    _ => unreachable!(),
                };
                
                let old_bit = (old_counter >> bit_position) & 1;
                let new_bit = (self.internal_counter >> bit_position) & 1;
                
                if old_bit == 1 && new_bit == 0 {
                    let current_tima = result.new_tima.unwrap_or(tima);
                    if current_tima == 0xFF {
                        result.new_tima = Some(tma);
                        result.timer_interrupt = true;
                    } else {
                        result.new_tima = Some(current_tima + 1);
                    }
                }
            }
        }
        
        result
    }

    pub fn reset_divider(&mut self) {
        self.internal_counter = 0;
    }
}