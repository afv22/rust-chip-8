use std::fs;

const CODEFILE: &str = "src/code.txt";

fn load_program(fp: &str) -> Vec<String> {
    let code = fs::read_to_string(fp).expect("File read failed!");
    return code.lines().map(|line| line.to_string()).collect();
}

pub struct VirtualMachine {
    instructions: Vec<String>,
    ip: i32,
}

impl VirtualMachine {

}

fn main() {
    let program = load_program(CODEFILE);
    let mut vm = VirtualMachine {
        instructions: program,
        ip: 0,
    };
}
