use super::{cpu::Interrupt, mmu::Mmu};

#[derive(Default)]
pub struct Joypad {
    right: bool,
    left: bool,
    up: bool,
    down: bool,
    a: bool,
    b: bool,
    select: bool,
    start: bool,
}

impl Joypad {
    pub fn new() -> Self {
        Self { 
            right: false, 
            left: false, 
            up: false, 
            down: false, 
            a: false, 
            b: false, 
            select: false, 
            start: false 
        }
    }

    fn is_pressed(&self, key: &Key) -> bool {
        match key {
            Key::Right  => self.right,
            Key::Left   => self.left,
            Key::Up     => self.up,
            Key::Down   => self.down,
            Key::A      => self.a,
            Key::B      => self.b,
            Key::Select => self.select,
            Key::Start  => self.start,
        }
    }

    fn set(&mut self, key: Key, pressed: bool) {
        match key {
            Key::Right => self.right = pressed,
            Key::Left  => self.left  = pressed,
            Key::Up    => self.up    = pressed,
            Key::Down  => self.down  = pressed,
            Key::A     => self.a     = pressed,
            Key::B     => self.b     = pressed,
            Key::Select=> self.select= pressed,
            Key::Start => self.start = pressed,
        }
    }

    pub fn press(&mut self, key: Key, mmu: &mut Mmu) {
        let was_pressed = self.is_pressed(&key);

        if !was_pressed {
            mmu.request_interrupt(Interrupt::Joypad);
        }

        self.set(key, true);
    }

    pub fn release(&mut self, key: Key) {
        self.set(key, false);
    }
}

pub enum Key {
    Right, Left, Up, Down, A, B, Select, Start
}