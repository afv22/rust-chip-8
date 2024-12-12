use crate::{drivers::display_driver::DisplayDriver, stack::Stack};
use std::{fs, usize};

const PROGRAM_START: usize = 0x200;
const MEMORY_SIZE: usize = 4096;
const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;

fn digit(hex: u16, place: u8) -> usize {
    let offset = place << 2;
    ((hex & (0xF << offset)) >> offset) as usize
}

pub struct Processor {
    memory: [u8; MEMORY_SIZE],
    stack: Stack<usize>,
    pixel_map: [[u8; SCREEN_WIDTH]; SCREEN_HEIGHT],
    _display: DisplayDriver,
    _keyboard: [u8; 16],
    pc: usize,
    v: [u8; 16],
    i: u16,
    delay_timer: u8,
    sound_timer: u8,
}

impl Processor {
    pub fn new() -> Processor {
        let display = DisplayDriver::new(&sdl2::init().unwrap());
        Processor {
            memory: [0; MEMORY_SIZE],
            stack: Stack::new(),
            pixel_map: [[0; SCREEN_WIDTH]; SCREEN_HEIGHT],
            _display: display,
            _keyboard: [0; 16],
            pc: PROGRAM_START,
            v: [0; 16],
            i: 0,
            delay_timer: 0,
            sound_timer: 0,
        }
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
        loop {
            let instruction =
                ((self.memory[self.pc] as u16) << 8) | self.memory[self.pc + 1] as u16;
            self.execute_instruction(instruction);
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
            }
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
            }
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
        self.pixel_map = [[0; SCREEN_WIDTH]; SCREEN_HEIGHT];
        self.step();
    }

    /// Return from a subroutine.
    fn op_00ee(&mut self, _i: u16) {
        self.pc = self.stack.pop().unwrap();
    }

    /// Jump to address NNN.
    fn op_1nnn(&mut self, i: u16) {
        self.pc = PROGRAM_START + (i as usize & 0xfff);
    }

    /// Execute subroutine starting at address NNN.
    fn op_2nnn(&mut self, i: u16) {
        self.stack.push(self.pc);
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
        self.v[(i >> 8) as usize & 0xf] += (i & 0xff) as u8;
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

    /// Display a sprite at position VX, VY with a width of 8 pixels and a height of N pixels.
    fn op_dxyn(&mut self, i: u16) {
        // TODO
        self.step();
    }

    /// Skip the following instruction if the key stored in register VX is pressed.
    fn op_ex9e(&mut self, i: u16) {
        // TODO
        self.step();
    }

    /// Skip the following instruction if the key stored in register VX isn't pressed.
    fn op_exa1(&mut self, i: u16) {
        // TODO
        self.step();
    }

    /// Set the value of register VX to the value of the delay timer.
    fn op_fx07(&mut self, i: u16) {
        self.v[(i >> 8) as usize & 0xf] = self.delay_timer;
        self.step();
    }

    /// Wait for a keypress and store the result in register VX
    fn op_fx0a(&mut self, i: u16) {
        // TODO
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
        // TODO
        self.step();
    }

    /// Store the binary-coded decimal representation of the value of register VX at addresses I, I+1, and I+2.
    fn op_fx33(&mut self, i: u16) {
        // TODO
        self.step();
    }

    /// Store the values of registers V0 to VX inclusive in memory starting at address I. I is set to I + X + 1 after operation
    fn op_fx55(&mut self, i: u16) {
        // TODO
        self.step();
    }

    /// Fill registers V0 to VX inclusive with values from memory starting at address I. I is set to I + X + 1 after operation
    fn op_fx65(&mut self, i: u16) {
        // TODO
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
