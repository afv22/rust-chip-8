mod drivers;
mod instruction;
mod processor;
mod stack;

const PROGRAM: &str = "./programs/pong.rom";

fn main() {
    let mut vm = processor::Processor::new();
    vm.load_program(PROGRAM);
    vm.execute();
}
