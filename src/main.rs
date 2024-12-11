use std::{fs, usize};
mod stack;
use stack::Stack;
mod instructions;
use instructions::Instruction;

const PROGRAM: &str = "./programs/pong.rom";
const PROGRAM_START: usize = 0x200;
const MEMORY_SIZE: usize = 4096;
const SCREEN_SIZE: usize = 64 * 32;

fn digit(hex: u16, place: u8) -> usize {
    ((hex & (0x1 << place)) >> place) as usize
}

pub struct VirtualMachine {
    memory: [u8; MEMORY_SIZE],
    _stack: Stack<u16>,
    _graphics: [u8; SCREEN_SIZE],
    _keyboard: [u8; 16],
    pc: usize,
    v: [u8; 16],
    _i: u16,
    _delay_timer: u8,
    _sound_times: u8,
}

impl VirtualMachine {
    fn new() -> VirtualMachine {
        VirtualMachine {
            memory: [0; MEMORY_SIZE],
            _stack: Stack::new(),
            _graphics: [0; SCREEN_SIZE],
            _keyboard: [0; 16],
            pc: PROGRAM_START,
            v: [0; 16],
            _i: 0,
            _delay_timer: 0,
            _sound_times: 0,
        }
    }

    fn step(&mut self) {
        self.pc += 2;
    }

    fn jump(&mut self, addr: u16) {
        self.pc = PROGRAM_START + addr as usize
    }

    fn load_program(&mut self, fp: &str) {
        let bytes = fs::read(fp).unwrap();
        for (pos, byte) in bytes.iter().enumerate() {
            self.memory[PROGRAM_START + pos] = *byte;
        }
    }

    fn resolve_instruction(&mut self, instruction: u16) -> Instruction {
        let vx = digit(instruction, 2);
        let vy = digit(instruction, 1);
        match instruction {
            x if x == 0x00e0 => Instruction::CLS(),
            x if x == 0x00ee => Instruction::RET(),
            x if digit(x, 3) == 0 => Instruction::NOP(),
            x if digit(x, 3) == 1 => Instruction::JMP(x & 0xfff),
            x if digit(x, 3) == 2 => Instruction::CALL(x & 0xfff),
            x if digit(x, 3) == 3 => Instruction::SE(vx, x as u8),
            x if digit(x, 3) == 4 => Instruction::SNE(vx, x as u8),
            x if digit(x, 3) == 5 => Instruction::SE(vx, self.v[vy]),
            x if digit(x, 3) == 6 => Instruction::SET(vx, x as u8),
            x if digit(x, 3) == 7 => Instruction::ADD(vx, x as u8),
            x if digit(x, 3) == 8 => match digit(x, 0) {
                0x0 => Instruction::SET(vx, self.v[vx]),
                0x1 => Instruction::ORV(vx, vy),
                0x2 => Instruction::ANDV(vx, vy),
                0x3 => Instruction::XORV(vx, vy),
                0x4 => Instruction::ADDV(vx, vy),
                0x5 => Instruction::SUBV(vx, vy),
                0x6 => Instruction::SHRV(vx),
                0x7 => Instruction::SUBNV(vx, vy),
                0xE => Instruction::SHLV(vx),
                _ => Instruction::INVALID(),
            },
            x if digit(x, 3) == 9 => Instruction::SNE(vx, self.v[vy]),
            _ => Instruction::INVALID(),
        }
    }

    fn execute(&mut self) {
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
                    self._stack.push(self.pc as u16);
                    self.pc = addr as usize;
                }
                Instruction::CLS() => self._graphics = [0; SCREEN_SIZE],
                Instruction::JMP(addr) => self.jump(addr),
                Instruction::ORV(x, y) => self.v[x] = self.v[x] | self.v[y],
                Instruction::RET() => self.pc = self._stack.pop().unwrap() as usize,
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
                Instruction::UNIMPLEMENTED() => println!("Instruction has not been implemented."),
                Instruction::INVALID() => println!("Invalid instruction!"),
            }
            self.step();
        }
    }
}

fn main() {
    let mut vm = VirtualMachine::new();
    vm.load_program(PROGRAM);
    vm.execute();
}
