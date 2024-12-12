use crate::{drivers::display_driver::DisplayDriver, instruction::Instruction, stack::Stack};
use rand::Rng;
use sdl2::libc::sleep;
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
    _delay_timer: u8,
    _sound_times: u8,
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
            _delay_timer: 0,
            _sound_times: 0,
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

    fn resolve_instruction(&mut self, instruction: u16) -> Instruction {
        let vx = digit(instruction, 2);
        let vy = digit(instruction, 1);
        match digit(instruction, 3) {
            0x0 => match instruction {
                0x00e0 => Instruction::CLS(),
                0x00ee => Instruction::RET(),
                _ => Instruction::NOP(),
            },
            0x1 => Instruction::JMP(instruction as usize & 0xfff),
            0x2 => Instruction::CALL(instruction as usize & 0xfff),
            0x3 => Instruction::SE(vx, instruction as u8),
            0x4 => Instruction::SNE(vx, instruction as u8),
            0x5 => match digit(instruction, 0) {
                0x0 => Instruction::SE(vx, self.v[vy]),
                _ => Instruction::INVALID(),
            },
            0x6 => Instruction::SET(vx, instruction as u8),
            0x7 => Instruction::ADD(vx, instruction as u8),
            0x8 => match digit(instruction, 0) {
                0x0 => Instruction::SET(vx, self.v[vx]),
                0x1 => Instruction::ORV(vx, vy),
                0x2 => Instruction::ANDV(vx, vy),
                0x3 => Instruction::XORV(vx, vy),
                0x4 => Instruction::ADDV(vx, vy),
                0x5 => Instruction::SUBV(vx, vy),
                0x6 => Instruction::SHRV(vx),
                0x7 => Instruction::SUBNV(vx, vy),
                0xe => Instruction::SHLV(vx),
                _ => Instruction::INVALID(),
            },
            0x9 => match digit(instruction, 0) {
                0x0 => Instruction::SNE(vx, self.v[vy]),
                _ => Instruction::INVALID(),
            },
            0xa => Instruction::LDI(instruction & 0xfff),
            0xb => Instruction::JMP(self.v[0] as usize + instruction as usize & 0xfff),
            0xc => Instruction::RND(digit(instruction, 2), instruction as u8),
            0xd => Instruction::DRW(
                digit(instruction, 2),
                digit(instruction, 1),
                digit(instruction, 0) as u8,
            ),
            0xe => match instruction as u8 {
                0x9e => Instruction::NOP(),
                0xa1 => Instruction::NOP(),
                _ => Instruction::INVALID(),
            },
            0xf => match instruction as u8 {
                0x07 => Instruction::SET(vx, self._delay_timer),
                0x0a => Instruction::NOP(),
                0x15 => Instruction::NOP(),
                0x18 => Instruction::NOP(),
                0x1e => Instruction::NOP(),
                0x29 => Instruction::NOP(),
                0x33 => Instruction::NOP(),
                0x55 => Instruction::NOP(),
                0x65 => Instruction::NOP(),
                _ => Instruction::INVALID(),
            },
            _ => Instruction::INVALID(),
        }
    }

    pub fn execute(&mut self) {
        loop {
            let instruction =
                ((self.memory[self.pc] as u16) << 8) | self.memory[self.pc + 1] as u16;

            match self.resolve_instruction(instruction) {
                Instruction::ADD(v, n) => self.v[v] += n,
                Instruction::ADDV(x, y) => {
                    let (val, carry) = self.v[x].overflowing_add(self.v[y]);
                    self.v[0xf] = if carry { 1 } else { 0 };
                    self.v[x] = val;
                }
                Instruction::ANDV(x, y) => self.v[x] = self.v[x] & self.v[y],
                Instruction::CALL(addr) => {
                    self.stack.push(self.pc);
                    self.pc = addr;
                    continue;
                }
                Instruction::CLS() => self.pixel_map = [[0; SCREEN_WIDTH]; SCREEN_HEIGHT],
                Instruction::DRW(x, y, n) => {}
                Instruction::JMP(addr) => {
                    self.pc = PROGRAM_START + addr;
                    continue;
                }
                Instruction::LDI(addr) => self.i = addr,
                Instruction::ORV(x, y) => self.v[x] = self.v[x] | self.v[y],
                Instruction::RND(x, k) => self.v[x] = rand::thread_rng().gen_range(0..=255) & k,
                Instruction::RET() => {
                    self.pc = self.stack.pop().unwrap();
                    continue;
                }
                Instruction::SE(x, y) => {
                    if self.v[x] == y {
                        self.step()
                    }
                }
                Instruction::SHRV(x) => {
                    self.v[0xf] = self.v[x] & 0x1;
                    self.v[x] >>= 1;
                }
                Instruction::SHLV(x) => {
                    self.v[0xf] = (self.v[x] & 0x80) >> 7;
                    self.v[x] <<= 1;
                }
                Instruction::SNE(x, y) => {
                    if self.v[x] != y {
                        self.step()
                    }
                }
                Instruction::SET(v, n) => self.v[v] = n,
                Instruction::SUBV(x, y) => {
                    let (val, carry) = self.v[x].overflowing_sub(self.v[y]);
                    self.v[0xf] = if carry { 1 } else { 0 };
                    self.v[x] = val;
                }
                Instruction::SUBNV(x, y) => {
                    let (val, carry) = self.v[y].overflowing_sub(self.v[x]);
                    self.v[0xf] = if carry { 1 } else { 0 };
                    self.v[x] = val;
                }
                Instruction::XORV(x, y) => self.v[x] = self.v[x] ^ self.v[y],
                Instruction::NOP() => {}
                Instruction::INVALID() => println!("Invalid instruction!"),
            }
            self.step();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        instruction::Instruction,
        processor::{Processor, PROGRAM_START},
    };

    #[test]
    fn test_load_program() {
        let mut vm = Processor::new();
        assert_eq!(vm.memory[PROGRAM_START], 0x0000);
        vm.load_program("./programs/pong.rom");
        assert_eq!(vm.memory[PROGRAM_START], 0x6a);
    }

    #[test]
    fn test_resolve_instruction() {
        let mut vm = Processor::new();
        assert_eq!(vm.resolve_instruction(0x0234), Instruction::NOP());
        assert_eq!(vm.resolve_instruction(0x1234), Instruction::JMP(0x234));
    }

    #[test]
    fn test_display() {}
}
