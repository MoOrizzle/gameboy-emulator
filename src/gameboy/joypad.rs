#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Key {
    Right  = 0,
    Left   = 1,
    Up     = 2,
    Down   = 3,
    A      = 4,
    B      = 5,
    Select = 6,
    Start  = 7,
}

enum KeyGroup {
    Directions,
    Buttons,
}

impl Key {
    fn group(self) -> KeyGroup {
        match self {
            Key::Right | Key::Left | Key::Up     | Key::Down  => KeyGroup::Directions,
            Key::A     | Key::B    | Key::Select | Key::Start => KeyGroup::Buttons,
        }
    }

    fn bit(self) -> u8 {
        (self as u8) % 4
    }
}

pub struct Joypad {
    select_buttons: bool,
    select_directions: bool,

    buttons: u8,    // Bit 0-3
    directions: u8, // Bit 0-3
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            select_buttons: false,
            select_directions: false,
            buttons: 0x0F,
            directions: 0x0F,
        }
    }

    fn target_mut(&mut self, key: Key) -> &mut u8 {
        match key.group() {
            KeyGroup::Directions => &mut self.directions,
            KeyGroup::Buttons    => &mut self.buttons,
        }
    }

    pub fn write(&mut self, value: u8) {
        self.select_buttons = value & 0x20 == 0;
        self.select_directions = value & 0x10 == 0;
    }

    pub fn read(&self) -> u8 {
        let mut result = 0xC0;

        if !self.select_buttons {
            result |= 0x20;
        }

        if !self.select_directions {
            result |= 0x10;
        }

        let low = if self.select_buttons {
            self.buttons
        } else if self.select_directions {
            self.directions
        } else {
            0x0F
        };

        result | (low & 0x0F)
    }

    pub fn press(&mut self, key: Key) -> bool {
        let mask = 1 << key.bit();
        let target = self.target_mut(key);

        let was_released = (*target & mask) != 0;
        *target &= !mask;

        was_released
    }

    pub fn release(&mut self, key: Key) {
        let mask = 1 << key.bit();
        *self.target_mut(key) |= mask;
    }
}