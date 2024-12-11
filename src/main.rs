use std::fs;

mod stack;
use stack::Stack;

const CODEFILE: &str = "src/code.txt";

pub struct Registers {
    _v0: i8,
    _v1: i8,
    _vf: i8,
    _i: usize,
    pc: usize,
}

pub struct VirtualMachine {
    instructions: Vec<String>,
    registers: Registers,
    _stack: Stack<usize>,
}

impl VirtualMachine {
    fn new(instructions: Vec<String>) -> VirtualMachine {
        VirtualMachine {
            instructions: instructions,
            registers: Registers {
                _v0: 0,
                _v1: 0,
                _vf: 0,
                _i: 0,
                pc: 0,
            },
            _stack: Stack::new(),
        }
    }

    fn execute(&mut self) {
        loop {
            let instruction = self.instructions[self.registers.pc].clone();
            println!("{:?}", instruction);
            self.registers.pc += 1;
        }
    }
}

fn load_program(fp: &str) -> Vec<String> {
    let code = fs::read_to_string(fp).expect("File read failed!");
    return code.lines().map(|line| line.to_string()).collect();
}

fn main() {
    let program = load_program(CODEFILE);
    let mut vm = VirtualMachine::new(program);
    vm.execute();
}
