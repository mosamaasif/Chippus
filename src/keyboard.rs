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
}
