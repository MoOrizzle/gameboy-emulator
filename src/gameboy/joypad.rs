#[derive(Copy, Clone)]
pub enum Key {
    Right = 0,
    Left  = 1,
    Up    = 2,
    Down  = 3,
    A     = 4,
    B     = 5,
    Select= 6,
    Start = 7,
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

    pub fn write(&mut self, value: u8) {
        self.select_buttons = value & 0x20 == 0;
        self.select_directions = value & 0x10 == 0;
    }

    pub fn read(&self) -> u8 {
        let mut result = 0xC0;

        if self.select_buttons {
            result |= 0x20;
        }
    
        if self.select_directions {
            result |= 0x10;
        }

        if self.select_buttons {
            result |= self.buttons & 0x0F;
        } else if self.select_directions {
            result |= self.directions & 0x0F;
        } else {
            result |= 0x0F;
        }

        result
    }

    pub fn press(&mut self, key: Key) -> bool {
        let (mask, target) = match key {
            Key::Right => (1 << 0, &mut self.directions),
            Key::Left  => (1 << 1, &mut self.directions),
            Key::Up    => (1 << 2, &mut self.directions),
            Key::Down  => (1 << 3, &mut self.directions),
            Key::A     => (1 << 0, &mut self.buttons),
            Key::B     => (1 << 1, &mut self.buttons),
            Key::Select=> (1 << 2, &mut self.buttons),
            Key::Start => (1 << 3, &mut self.buttons),
        };

        let was_released = *target & mask != 0;
        *target &= !mask;

        was_released
    }

    pub fn release(&mut self, key: Key) {
        let (mask, target) = match key {
            Key::Right => (1 << 0, &mut self.directions),
            Key::Left  => (1 << 1, &mut self.directions),
            Key::Up    => (1 << 2, &mut self.directions),
            Key::Down  => (1 << 3, &mut self.directions),
            Key::A     => (1 << 0, &mut self.buttons),
            Key::B     => (1 << 1, &mut self.buttons),
            Key::Select=> (1 << 2, &mut self.buttons),
            Key::Start => (1 << 3, &mut self.buttons),
        };

        *target |= mask;
    }
}