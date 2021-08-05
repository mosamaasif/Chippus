use rand::Rng;
use std::fs;
use std::path::PathBuf;

use crate::emulator::keyboard::Keyboard;
use crate::emulator::screen::Screen;

pub struct Emulator {
    pub ram: [u8; 4096], // The actual Memory or RAM
    pub stack: Vec<u16>, // 16 entires of 16 bits each
    pub v: [u8; 16],     // 8-bit 16 registers (v0-vF)
    pub i: u16,          // The 16-bit Index Register
    pub pc: u16,         // 16-bit program counter
    pub delay_timer: u8, // 8-bit delay timer
    pub sound_timer: u8, // 8-bit sound timer,
    total_dt: f32,       // total delay timer value

    pub screen: Screen,     // screen structure
    pub keyboard: Keyboard, // keyboard structure
    pause: bool,            // a way to pause emulator,
    rom_len: usize,         // size of rom loaded into memory or length of code
}

impl Emulator {
    pub fn new() -> Emulator {
        let fonts = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];

        let mut mem = [0u8; 4096];
        mem[(0x050 as usize)..=(0x09F as usize)].copy_from_slice(&fonts);

        Emulator {
            pc: 0x200 as u16,
            ram: mem,
            stack: Vec::new(),
            v: [0; 16],
            i: 0,
            delay_timer: 0,
            sound_timer: 0,
            screen: Screen::new(),
            keyboard: Keyboard::new(),
            pause: true,
            rom_len: 0,
            total_dt: 0.0f32,
        }
    }

    fn fetch_instruction(&self) -> u16 {
        (self.ram[(self.pc as usize)] as u16) << 8 | (self.ram[(self.pc as usize)] as u16)
    }

    fn execute_instruction(&mut self, instruction: u16) {
        let nibbles = (
            ((instruction & 0xF000) >> 12) as u8,
            ((instruction & 0x0F00) >> 8) as u8,
            ((instruction & 0x00F0) >> 4) as u8,
            (instruction & 0x000F) as u8,
        );

        self.pc += 2;

        match nibbles {
            (0x0, 0x0, 0xE, _) => {
                match nibbles.3 {
                    // This is used to clear the screen (CLS) (00E0)
                    0 => self.screen.clear(),
                    // Returns from a subroutine (RET) (00EE)
                    0xE => {
                        if let Some(address) = self.stack.pop() {
                            self.pc = address;
                        }
                    }
                    _ => (),
                }
            }

            // Jump to address nnn (JP addr) (1nnn)
            (0x1, _, _, _) => {
                self.pc = instruction & 0x0FFF;
            }

            // calls subroutine at address nnn(CALL addr) (2nnn)
            (0x2, _, _, _) => {
                self.stack.push(self.pc);
                self.pc = instruction & 0x0FFF;
            }

            // if value at register x is equal to kk, then skip next instruction (SE Vx, byte) (3xkk)
            (0x3, _, _, _) => {
                if self.v[nibbles.1 as usize] == ((instruction & 0x00FF) as u8) {
                    self.pc += 2;
                }
            }

            // if value at register x is not equal to kk, then skip next instruction (SNE Vx, byte) (4xkk)
            (0x4, _, _, _) => {
                if self.v[nibbles.1 as usize] != ((instruction & 0x00FF) as u8) {
                    self.pc += 2;
                }
            }

            // if value at register x is equal to register y, then skip next instruction (SE Vx, Vy) (5xy0)
            (0x5, _, _, 0x0) => {
                if self.v[nibbles.1 as usize] == self.v[nibbles.2 as usize] {
                    self.pc += 2;
                }
            }

            // put value kk into register x (LD Vx, byte) (6xkk)
            (0x6, _, _, _) => {
                self.v[nibbles.1 as usize] = (instruction & 0x00FF) as u8;
            }

            // add value kk into register x (ADD Vx, byte) (7xkk)
            (0x7, _, _, _) => {
                let x = nibbles.1 as usize;
                self.v[x] = self.v[x].wrapping_add((instruction & 0x00FF) as u8);
            }

            (0x8, _, _, _) => {
                match nibbles.3 {
                    // set value of register Vy into Vx (LD Vx, Vy) (8xy0)
                    0x0 => self.v[nibbles.1 as usize] = self.v[nibbles.2 as usize],
                    // OR value of register Vy with Vx and set in Vx (OR Vx, Vy) (8xy1)
                    0x1 => self.v[nibbles.1 as usize] |= self.v[nibbles.2 as usize],
                    // AND value of register Vy with Vx and set in Vx (AND Vx, Vy) (8xy2)
                    0x2 => self.v[nibbles.1 as usize] &= self.v[nibbles.1 as usize],
                    // XOR value of register Vy with Vx and set in Vx (XOR Vx, Vy) (8xy3)
                    0x3 => self.v[nibbles.1 as usize] ^= self.v[nibbles.2 as usize],
                    // ADD value of register Vy into Vx and set in Vx (ADD Vx, Vy) (8xy4)
                    0x4 => {
                        let x = nibbles.1 as usize;
                        let y = nibbles.2 as usize;
                        let (res, overflow) = self.v[x].overflowing_add(self.v[y]);

                        self.v[x] = res;
                        self.v[0xF] = overflow as u8;
                    }
                    // SUB value of register Vy from Vx and set in Vx (SUB Vx, Vy) (8xy5)
                    0x5 => {
                        let x = nibbles.1 as usize;
                        let y = nibbles.2 as usize;
                        let (res, overflow) = self.v[x].overflowing_sub(self.v[y]);

                        self.v[x] = res;
                        self.v[0xF] = !overflow as u8;
                    }
                    // Shift right Vx by 1 (SHR Vx) {, Vy} (8xy6)
                    0x6 => {
                        let x = nibbles.1 as usize;
                        self.v[0xF] = self.v[x] & 1;
                        self.v[x] >>= 1;
                    }
                    // SUB value of register Vx from Vy and set in Vx (SUBN Vx, Vy) (8xy7)
                    0x7 => {
                        let x = nibbles.1 as usize;
                        let y = nibbles.2 as usize;
                        let (res, overflow) = self.v[y].overflowing_sub(self.v[x]);

                        self.v[x] = res;
                        self.v[0xF] = !overflow as u8;
                    }
                    // Shift left Vx by 1 (SHL Vx) {, Vy} (8xyE)
                    0xE => {
                        let x = nibbles.1 as usize;
                        self.v[0xF] = self.v[x] & 0x80;
                        self.v[x] <<= 1;
                    }
                    _ => (),
                }
            }

            (0x9, _, _, 0x0) => {
                if self.v[nibbles.1 as usize] != self.v[nibbles.2 as usize] {
                    self.pc += 2;
                }
            }

            // set i to nnn (LD I, addr) (Annn)
            (0xA, _, _, _) => {
                self.i = instruction & 0x0FFF;
            }

            // jump to v0 + nnn (JP v0, addr) (Annn)
            (0xB, _, _, _) => {
                self.pc = (instruction & 0x0FFF) + (self.v[0] as u16);
            }

            // random value AND kk and set value in Vx register (RNG Vx, byte) (Cxkk)
            (0xC, _, _, _) => {
                let mut rng = rand::thread_rng();
                self.v[nibbles.1 as usize] = rng.gen::<u8>() & ((instruction & 0x00FF) as u8);
            }

            // display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision (DRW Vx, Vy, nibble) (Dxyn)
            (0xD, _, _, _) => {
                let x_coord = self.v[nibbles.1 as usize] as usize;
                let y_coord = self.v[nibbles.2 as usize] as usize;

                self.v[0xF] = self.screen.draw(
                    (x_coord, y_coord),
                    &self.ram[(self.i as usize)..((self.i + (nibbles.3 as u16)) as usize)],
                ) as u8;
            }

            (0xE, _, _, _) => {
                match (nibbles.2, nibbles.3) {
                    // if key with value Vx is pressed, skip next instruction (SKP Vx) (Ex9E)
                    (0x9, 0xE) => {
                        if self
                            .keyboard
                            .is_key_pressed(self.v[nibbles.1 as usize] as usize)
                        {
                            self.pc += 2;
                        }
                    }
                    // if key with value Vx is not pressed, skip next instruction (SKP Vx) (Ex9E)
                    (0xA, 0x1) => {
                        if !self
                            .keyboard
                            .is_key_pressed(self.v[nibbles.1 as usize] as usize)
                        {
                            self.pc += 2;
                        }
                    }
                    _ => (),
                }
            }

            (0xF, _, _, _) => {
                match (nibbles.2, nibbles.3) {
                    // value of delay timer in  Vx (LD Vx, DT) (Fx07)
                    (0x0, 0x7) => self.v[nibbles.1 as usize] = self.delay_timer,
                    // wait until key pressed, and store in  Vx (LD Vx, K) (Fx0A)
                    (0x0, 0xA) => {
                        if let Some(key) = self.keyboard.get_pressed_key() {
                            self.v[nibbles.1 as usize] = key;
                        } else {
                            self.pc -= 2; // move it back, so it stays on this instruction without halting thread
                        }
                    }
                    // set delay timer value equal to Vx (LD DT, Vx) (Fx15)
                    (0x1, 0x5) => {
                        self.delay_timer = self.v[nibbles.1 as usize];
                    }
                    // set sound timer equal to Vx (LD ST, Vx) (Fx18)
                    (0x1, 0x8) => {
                        self.sound_timer = self.v[nibbles.1 as usize];
                    }

                    // set i to i + Vx (ADD I, Vx) (Fx1E)
                    (0x1, 0xE) => {
                        self.i += self.v[nibbles.1 as usize] as u16;
                    }

                    // set i equal to location of sprite = Vx value (ADD I, Vx) (Fx29)
                    (0x2, 0x9) => {
                        self.i = (5 * self.v[nibbles.1 as usize]) as u16;
                    }

                    // store BCD representation of Vx in memory locations I, I+1, and I+2 (LD B, Vx) (Fx33)
                    (3, 3) => {
                        let x = nibbles.1 as usize;
                        self.ram[self.i as usize] = self.v[x] / 100;
                        self.ram[(self.i as usize) + 1] = (self.v[x] / 10) % 10;
                        self.ram[(self.i as usize) + 2] = self.v[x] % 10;
                    }

                    // store register V0 to Vx values in memory starting from location at reg (LD[I], Vx) (Fx55)
                    (0x5, 0x5) => {
                        let x = nibbles.1 as u16;
                        self.ram[(self.i as usize)..=((self.i + x) as usize)]
                            .copy_from_slice(&self.v[0..=(x as usize)]);
                        self.i += x + 1;
                    }

                    // store register V0 to Vx equal to values in memory starting from location at I (LD Vx, [I]) (Fx56)
                    (0x5, 0x6) => {
                        let x = nibbles.1 as u16;
                        self.v[0..=(x as usize)].copy_from_slice(
                            &self.ram[(self.i as usize)..=((self.i + x) as usize)],
                        );
                        self.i += x + 1;
                    }
                    _ => (),
                }
            }

            // To enter a subroutine (SYS addr) (0nnn)
            (0, _, _, _) => {
                // ignoring 0nnn, this is for older computer.
            }

            _ => {}
        }
    }

    pub fn execute_cycle(&mut self, dt: f32) {
        if !self.pause {
            self.update_timer(dt);
            // fetch instruction from memory
            let instruction = self.fetch_instruction();

            // decode and execute instruction
            self.execute_instruction(instruction);
        }
    }

    pub fn load_rom(&mut self, romfile: &PathBuf) {
        // Reset emulator
        *self = Self::new();

        // Load ROM from file
        let contents = match fs::read(romfile) {
            Err(e) => {
                println!(
                    "Failed to read file: '{0}', [ERROR]: {1}",
                    romfile.display(),
                    e
                );
                std::process::exit(0)
            }
            Ok(f) => f,
        };

        // Copy rom in memory
        self.ram[(0x200 as usize)..(0x200 + contents.len() as usize)]
            .copy_from_slice(&contents[..]);
        self.rom_len = contents.len();

        self.pause = false;
    }

    pub fn code_memory_location(&self) -> (usize, usize) {
        (0x200, 0x200 + self.rom_len)
    }

    fn update_timer(&mut self, dt: f32) {
        if self.delay_timer > 0 {
            self.total_dt += dt;
            const TIMER_PERIOD: f32 = 1.0 / 60.0;
            while self.total_dt > TIMER_PERIOD {
                self.total_dt -= TIMER_PERIOD;
                self.delay_timer -= 1;
            }
        }
    }
}
