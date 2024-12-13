use crate::drivers::{AudioDriver, DisplayDriver};
use sdl2::{event::Event, keyboard::Keycode};
use std::{
    fs, process, thread,
    time::{Duration, Instant},
    usize,
};

const PROGRAM_START: usize = 0x200;
const MEMORY_SIZE: usize = 4096;
const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const SPRITE_START: usize = 0x50;
const FRAME_RATE: f64 = 120.;

pub const FONT_SPRITES: [u8; 80] = [
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

fn digit(hex: u16, place: u8) -> usize {
    let offset = place << 2;
    ((hex & (0xF << offset)) >> offset) as usize
}

fn parse_key(key: Keycode) -> u8 {
    match key {
        Keycode::Num1
        | Keycode::Num2
        | Keycode::Num3
        | Keycode::Num4
        | Keycode::Num5
        | Keycode::Num6
        | Keycode::Num7
        | Keycode::Num8
        | Keycode::Num9
        | Keycode::Num0 => key.into_i32() as u8 - 48,
        Keycode::A | Keycode::B | Keycode::C | Keycode::D | Keycode::E | Keycode::F => {
            key.into_i32() as u8 - 87
        }
        _ => 0xff,
    }
}

pub struct Processor {
    memory: [u8; MEMORY_SIZE],
    stack: [usize; 16],
    display: [[u8; SCREEN_WIDTH]; SCREEN_HEIGHT],
    display_change: bool,
    keyboard: [u8; 16],
    pc: usize,
    sp: usize,
    v: [u8; 16],
    i: u16,
    delay_timer: u8,
    sound_timer: u8,
}

impl Processor {
    pub fn new() -> Self {
        let mut p = Self {
            memory: [0; MEMORY_SIZE],
            stack: [0; 16],
            display: [[0; SCREEN_WIDTH]; SCREEN_HEIGHT],
            display_change: false,
            keyboard: [0; 16],
            pc: PROGRAM_START,
            sp: 0,
            v: [0; 16],
            i: 0,
            delay_timer: 0,
            sound_timer: 0,
        };

        // Load the font sprites into memory
        for (pos, sprite) in FONT_SPRITES.iter().enumerate() {
            p.memory[SPRITE_START + pos] = *sprite;
        }

        p
    }

    fn step(&mut self) {
        self.pc += 2;
    }

    pub fn load_program(&mut self, fp: &str) {
        let bytes = fs::read(fp).unwrap();
        for (pos, byte) in bytes.iter().enumerate() {
            self.memory[PROGRAM_START + pos] = *byte;
        }
    }

    pub fn run_program(&mut self) {
        let target_frame_duration = Duration::from_secs_f64(1. / FRAME_RATE);

        let sdl_context = sdl2::init().unwrap();
        let mut display_driver = DisplayDriver::new(&sdl_context);
        let audio_driver = AudioDriver::new(&sdl_context, 480.0, 0.25).unwrap();

        loop {
            let frame_start = Instant::now();

            let instruction =
                ((self.memory[self.pc] as u16) << 8) | self.memory[self.pc + 1] as u16;
            self.execute_instruction(instruction);

            self.handle_events(&sdl_context);
            if self.display_change {
                display_driver.draw(&self.display);
                self.display_change = false;
            }

            if self.delay_timer > 0 {
                self.delay_timer -= 1;
            }

            if self.sound_timer > 0 {
                audio_driver.start();
                self.sound_timer -= 1;
            } else if audio_driver.is_active() {
                audio_driver.stop();
            }

            // Maintain a standard frame rate
            let elapsed = frame_start.elapsed();
            if let Some(remaining) = target_frame_duration.checked_sub(elapsed) {
                thread::sleep(remaining);
            }
        }
    }

    pub fn handle_events(&mut self, sdl_context: &sdl2::Sdl) {
        for event in sdl_context.event_pump().unwrap().poll_iter() {
            match event {
                Event::Quit { .. } => {
                    process::exit(0);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    process::exit(0);
                }
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    let key = parse_key(key);
                    if key == 0xff {
                        continue;
                    }
                    // Only stores one key at a time. Could update this to support multiple
                    self.keyboard = [0; 16];
                    self.keyboard[key as usize] = 1;
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    let key = parse_key(key);
                    if key == 0xff {
                        continue;
                    }
                    self.keyboard = [0; 16];
                }
                _ => {}
            }
        }
    }

    fn execute_instruction(&mut self, instruction: u16) {
        match digit(instruction, 3) {
            0x0 => match instruction {
                0x00e0 => self.op_00e0(instruction),
                0x00ee => self.op_00ee(instruction),
                _ => self.op_0nnn(instruction),
            },
            0x1 => self.op_1nnn(instruction),
            0x2 => self.op_2nnn(instruction),
            0x3 => self.op_3xkk(instruction),
            0x4 => self.op_4xkk(instruction),
            0x5 => match digit(instruction, 0) {
                0x0 => self.op_5xy0(instruction),
                _ => {}
            },
            0x6 => self.op_6xkk(instruction),
            0x7 => self.op_7xkk(instruction),
            0x8 => match digit(instruction, 0) {
                0x0 => self.op_8xy0(instruction),
                0x1 => self.op_8xy1(instruction),
                0x2 => self.op_8xy2(instruction),
                0x3 => self.op_8xy3(instruction),
                0x4 => self.op_8xy4(instruction),
                0x5 => self.op_8xy5(instruction),
                0x6 => self.op_8xy6(instruction),
                0x7 => self.op_8xy7(instruction),
                0xe => self.op_8xye(instruction),
                _ => {}
            },
            0x9 => match digit(instruction, 0) {
                0x0 => self.op_9xy0(instruction),
                _ => {}
            },
            0xa => self.op_annn(instruction),
            0xb => self.op_bnnn(instruction),
            0xc => self.op_cxkk(instruction),
            0xd => self.op_dxyn(instruction),
            0xe => match instruction as u8 {
                0x9e => self.op_ex9e(instruction),
                0xa1 => self.op_exa1(instruction),
                _ => {}
            },
            0xf => match instruction as u8 {
                0x07 => self.op_fx07(instruction),
                0x0a => self.op_fx0a(instruction),
                0x15 => self.op_fx15(instruction),
                0x18 => self.op_fx18(instruction),
                0x1e => self.op_fx1e(instruction),
                0x29 => self.op_fx29(instruction),
                0x33 => self.op_fx33(instruction),
                0x55 => self.op_fx55(instruction),
                0x65 => self.op_fx65(instruction),
                _ => {}
            },
            _ => {}
        }
    }

    /// (DEPRECATED) Execute a machine code subroutine at address NNN.
    fn op_0nnn(&mut self, _i: u16) {
        self.step();
    }

    /// Clear the screen.
    fn op_00e0(&mut self, _i: u16) {
        self.display = [[0; SCREEN_WIDTH]; SCREEN_HEIGHT];
        self.display_change = true;
        self.step();
    }

    /// Return from a subroutine.
    fn op_00ee(&mut self, _i: u16) {
        self.sp -= 1;
        self.pc = self.stack[self.sp];
        self.step();
    }

    /// Jump to address NNN.
    fn op_1nnn(&mut self, i: u16) {
        self.pc = i as usize & 0xfff;
    }

    /// Execute subroutine starting at address NNN.
    fn op_2nnn(&mut self, i: u16) {
        self.stack[self.sp] = self.pc;
        self.sp += 1;
        self.pc = i as usize & 0xfff;
    }

    /// Skip the following instruction if the value of register VX equals KK.
    fn op_3xkk(&mut self, i: u16) {
        if self.v[(i >> 8) as usize & 0xf] == (i & 0xff) as u8 {
            self.step();
        }
        self.step();
    }

    /// Skip the following instruction if the value of register VX doesn't equal KK.
    fn op_4xkk(&mut self, i: u16) {
        if self.v[(i >> 8) as usize & 0xf] != (i & 0xff) as u8 {
            self.step();
        }
        self.step();
    }

    /// Skip the following instruction if the value of register VX equals the value of register VY.
    fn op_5xy0(&mut self, i: u16) {
        if self.v[(i >> 8) as usize & 0xf] == self.v[(i >> 4) as usize & 0xf] {
            self.step();
        }
        self.step();
    }

    /// Store the value KK in register VX.
    fn op_6xkk(&mut self, i: u16) {
        self.v[(i >> 8) as usize & 0xf] = (i & 0xff) as u8;
        self.step();
    }

    /// Add the value KK to the value of register VX.
    fn op_7xkk(&mut self, i: u16) {
        let v = (i >> 8) as usize & 0xf;
        self.v[v] = self.v[v].wrapping_add((i & 0xff) as u8);
        self.step();
    }

    /// Store the value of register VY in register VX.
    fn op_8xy0(&mut self, i: u16) {
        self.v[(i >> 8) as usize & 0xf] = self.v[(i >> 4) as usize & 0xf];
        self.step();
    }

    /// Set the value of register VX to the value of register VX OR the value of register VY.
    fn op_8xy1(&mut self, i: u16) {
        self.v[(i >> 8) as usize & 0xf] |= self.v[(i >> 4) as usize & 0xf];
        self.step();
    }

    /// Set the value of register VX to the value of register VX AND the value of register VY.
    fn op_8xy2(&mut self, i: u16) {
        self.v[(i >> 8) as usize & 0xf] &= self.v[(i >> 4) as usize & 0xf];
        self.step();
    }

    /// Set the value of register VX to the value of register VX XOR the value of register VY.
    fn op_8xy3(&mut self, i: u16) {
        self.v[(i >> 8) as usize & 0xf] ^= self.v[(i >> 4) as usize & 0xf];
        self.step();
    }

    /// Add the value of register VY to the value of register VX.
    fn op_8xy4(&mut self, i: u16) {
        let (result, carry) =
            self.v[(i >> 8) as usize & 0xf].overflowing_add(self.v[(i >> 4) as usize & 0xf]);
        self.v[(i >> 8) as usize & 0xf] = result;
        self.v[0xf] = carry as u8;
        self.step();
    }

    /// Subtract the value of register VY from the value of register VX.
    fn op_8xy5(&mut self, i: u16) {
        let (result, carry) =
            self.v[(i >> 8) as usize & 0xf].overflowing_sub(self.v[(i >> 4) as usize & 0xf]);
        self.v[(i >> 8) as usize & 0xf] = result;
        self.v[0xf] = !carry as u8;
        self.step();
    }

    /// Store the value of register VY shifted right one bit in register VX.
    fn op_8xy6(&mut self, i: u16) {
        self.v[0xf] = self.v[(i >> 8) as usize & 0xf] & 0x1;
        self.v[(i >> 8) as usize & 0xf] >>= 1;
        self.step();
    }

    /// Set the value of register VX to the value of register VY minus the value of register VX.
    fn op_8xy7(&mut self, i: u16) {
        let (result, carry) =
            self.v[(i >> 4) as usize & 0xf].overflowing_sub(self.v[(i >> 8) as usize & 0xf]);
        self.v[(i >> 8) as usize & 0xf] = result;
        self.v[0xf] = !carry as u8;
        self.step();
    }

    /// Store the value of register VY shifted left one bit in register VX.
    fn op_8xye(&mut self, i: u16) {
        self.v[0xf] = self.v[(i >> 8) as usize & 0xf] >> 7;
        self.v[(i >> 8) as usize & 0xf] <<= 1;
        self.step();
    }

    /// Skip the following instruction if the value of register VX doesn't equal the value of register VY.
    fn op_9xy0(&mut self, i: u16) {
        if self.v[(i >> 8) as usize & 0xf] != self.v[(i >> 4) as usize & 0xf] {
            self.step();
        }
        self.step();
    }

    /// Store address NNN in register I.
    fn op_annn(&mut self, i: u16) {
        self.i = i & 0xfff;
        self.step();
    }

    /// Jump to address NNN + V0.
    fn op_bnnn(&mut self, i: u16) {
        self.pc = (i & 0xfff) as usize + self.v[0] as usize;
    }

    /// Set VX to a random number with a mask of NN (0 to 255).
    fn op_cxkk(&mut self, i: u16) {
        self.v[(i >> 8) as usize & 0xf] = rand::random::<u8>() & (i & 0xff) as u8;
        self.step();
    }

    /// Display the sprite stored at the address held in register I at
    /// position VX, VY with a width of 8 pixels and a height of N pixels.
    fn op_dxyn(&mut self, i: u16) {
        let x = self.v[(i >> 8) as usize & 0xf] as usize;
        let y = self.v[(i >> 4) as usize & 0xf] as usize;
        self.v[0xf] = 0;

        for row in 0..(i & 0xf) as usize {
            let sprite_byte = self.memory[self.i as usize + row];
            for col in 0..8 {
                let sprite_pixel = sprite_byte >> (7 - col) & 0x1;
                let x_coord = (x + col) % SCREEN_WIDTH;
                let y_coord = (y + row) % SCREEN_HEIGHT;
                if sprite_pixel == 1 {
                    if self.display[y_coord][x_coord] == 1 {
                        self.v[0xf] = 1;
                    }
                    self.display[y_coord][x_coord] ^= 1;
                }
            }
        }
        self.display_change = true;
        self.step();
    }

    /// Skip the following instruction if the key stored in register VX is pressed.
    fn op_ex9e(&mut self, i: u16) {
        if self.keyboard[self.v[(i >> 8) as usize & 0xf] as usize] == 1 {
            self.step();
        }
        self.step();
    }

    /// Skip the following instruction if the key stored in register VX isn't pressed.
    fn op_exa1(&mut self, i: u16) {
        if self.keyboard[self.v[(i >> 8) as usize & 0xf] as usize] == 1 {
            self.step();
        }
        self.step();
    }

    /// Set the value of register VX to the value of the delay timer.
    fn op_fx07(&mut self, i: u16) {
        self.v[(i >> 8) as usize & 0xf] = self.delay_timer;
        self.step();
    }

    /// Wait for a keypress and store the result in register VX
    fn op_fx0a(&mut self, i: u16) {
        // wait for a key to be pressed, and store the value in register VX
        if self.keyboard.iter().all(|&x| x == 0) {
            return;
        }
        let key = self.keyboard.iter().position(|&x| x == 1);
        self.v[(i >> 8) as usize & 0xf] = key.unwrap() as u8;
        self.step();
    }

    /// Set the delay timer to the value of register VX.
    fn op_fx15(&mut self, i: u16) {
        self.delay_timer = self.v[(i >> 8) as usize & 0xf];
        self.step();
    }

    /// Set the sound timer to the value of register VX.
    fn op_fx18(&mut self, i: u16) {
        self.sound_timer = self.v[(i >> 8) as usize & 0xf];
        self.step();
    }

    /// Add the value of register VX to the value of register I.
    fn op_fx1e(&mut self, i: u16) {
        self.i += self.v[(i >> 8) as usize & 0xf] as u16;
        self.step();
    }

    /// Set I to the location of the sprite for the character in register VX.
    fn op_fx29(&mut self, i: u16) {
        self.i = self.v[(i >> 8) as usize & 0xf] as u16 * 5;
        self.step();
    }

    /// Store the binary-coded decimal representation of the value of register VX at addresses I, I+1, and I+2.
    fn op_fx33(&mut self, i: u16) {
        let x = self.v[(i >> 8) as usize & 0xf];
        self.memory[self.i as usize] = x / 100;
        self.memory[self.i as usize + 1] = (x / 10) % 10;
        self.memory[self.i as usize + 2] = x % 10;
        self.step();
    }

    /// Store the values of registers V0 to VX inclusive in memory starting at address I. I is set to I + X + 1 after operation
    fn op_fx55(&mut self, i: u16) {
        for x in 0..((i >> 8) as usize & 0xf) {
            self.memory[self.i as usize + x] = self.v[x];
            self.i += 1;
        }
        self.step();
    }

    /// Fill registers V0 to VX inclusive with values from memory starting at address I. I is set to I + X + 1 after operation
    fn op_fx65(&mut self, i: u16) {
        for x in 0..((i >> 8) as usize & 0xf) {
            self.v[x] = self.memory[self.i as usize + x];
            self.i += 1;
        }
        self.step();
    }
}

#[cfg(test)]
mod tests {
    use crate::processor::{Processor, PROGRAM_START};

    #[test]
    fn test_load_program() {
        let mut vm = Processor::new();
        assert_eq!(vm.memory[PROGRAM_START], 0x0000);
        vm.load_program("./programs/pong.rom");
        assert_eq!(vm.memory[PROGRAM_START], 0x6a);
    }

    #[test]
    fn test_resolve_instruction() {}

    #[test]
    fn test_display() {}
}
