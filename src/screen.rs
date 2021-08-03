pub struct Screen {
    pub buffer: Vec<u8>,
    dirty: bool,
}

impl Screen {
    const WIDTH: usize = 64;
    const HEIGHT: usize = 32;

    pub fn new() -> Screen {
        let mut screen = Screen {
            buffer: Vec::new(),
            dirty: true,
        };

        screen.buffer.resize(Screen::WIDTH * Screen::HEIGHT, 0);

        screen
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.set_dirty(true);
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> u8 {
        self.buffer[y * Screen::WIDTH + x]
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, value: bool) {
        self.buffer[y * Screen::WIDTH + x] = value as u8;
        self.dirty = true;
    }

    pub fn draw(&mut self, coords: (usize, usize), sprite_data: &[u8]) -> bool {
        let rows = sprite_data.len();
        let mut collision = false;
        for j in 0..rows {
            let row = sprite_data[j];
            for i in 0..8 {
                let new_value = row >> (7 - i) & 0x01;
                if new_value == 1 {
                    let x_new = (coords.0 + i) % Screen::WIDTH;
                    let y_new = (coords.1 + j) % Screen::HEIGHT;
                    let old_value = self.get_pixel(x_new, y_new);
                    if old_value == 1 {
                        collision = true;
                    }
                    self.set_pixel(x_new, y_new, (new_value == 1) ^ (old_value == 1));
                }
            }
        }
        collision
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn set_dirty(&mut self, value: bool) {
        self.dirty = value;
    }
}
