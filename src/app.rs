use crate::chip8::Emulator;

pub struct Application {
    emulator: Emulator,
}

impl Application {
    pub fn new() -> Application {
        Application {
            emulator: Emulator::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            egui
        }
    }
}
