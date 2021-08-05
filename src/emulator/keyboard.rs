use winit::event::VirtualKeyCode;

pub struct Keyboard {
    keys: [bool; 16],
}

impl Keyboard {
    pub fn new() -> Keyboard {
        Keyboard { keys: [false; 16] }
    }

    pub fn is_key_pressed(&self, key: usize) -> bool {
        self.keys[key]
    }

    pub fn get_pressed_key(&self) -> Option<u8> {
        for (i, key) in self.keys.iter().enumerate() {
            if *key {
                return Some(i as u8);
            }
        }
        None
    }

    pub fn set(&mut self, key: usize, pressed: bool) {
        self.keys[key] = pressed;
    }

    pub fn map_key(code: VirtualKeyCode) -> usize {
        match code {
            VirtualKeyCode::Key1 => 0,
            VirtualKeyCode::Key2 => 1,
            VirtualKeyCode::Key3 => 2,
            VirtualKeyCode::Key4 => 3,
            VirtualKeyCode::Q => 4,
            VirtualKeyCode::W => 5,
            VirtualKeyCode::E => 6,
            VirtualKeyCode::R => 7,
            VirtualKeyCode::A => 8,
            VirtualKeyCode::S => 9,
            VirtualKeyCode::D => 10,
            VirtualKeyCode::F => 11,
            VirtualKeyCode::Z => 12,
            VirtualKeyCode::X => 13,
            VirtualKeyCode::C => 14,
            VirtualKeyCode::V => 15,
            _ => return usize::default(),
        }
    }
}
