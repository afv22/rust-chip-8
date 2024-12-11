use std::fs;
mod stack;
use stack::Stack;

const PROGRAM: &str = "./programs/pong.rom";
const PROGRAM_START: usize = 0x200;
const MEMORY_SIZE: usize = 4096;
const SCREEN_SIZE: usize = 2048;

pub struct VirtualMachine {
    memory: [u8; MEMORY_SIZE],
    _stack: Stack<u16>,
    _gfx: [u8; SCREEN_SIZE],
    _key: [u8; 16],
    pc: usize,
    _v: [u8; 16],
    _i: u16,
    _delay_timer: u8,
    _sound_times: u8,
}

impl VirtualMachine {
    fn new() -> VirtualMachine {
        VirtualMachine {
            memory: [0; MEMORY_SIZE],
            _stack: Stack::new(),
            _gfx: [0; SCREEN_SIZE],
            _key: [0; 16],
            pc: PROGRAM_START,
            _v: [0; 16],
            _i: 0,
            _delay_timer: 0,
            _sound_times: 0,
        }
    }

    fn load_program(&mut self, fp: &str) {
        let bytes = fs::read(fp).unwrap();
        // Make sure endianness isn't fucked
        for (pos, byte) in bytes.iter().enumerate() {
            self.memory[PROGRAM_START + pos] = *byte;
        }
    }

    fn execute(&mut self) {
        loop {
            let instruction = self.memory[self.pc];
            if instruction == 0 {
                break;
            }
            println!("{:#02x}", instruction);
            self.pc += 1;
        }
    }
}

fn main() {
    let mut vm = VirtualMachine::new();
    vm.load_program(PROGRAM);
    vm.execute();
}
