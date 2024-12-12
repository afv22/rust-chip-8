mod drivers;
mod processor;
mod stack;

const PROGRAM: &str = "./programs/Breakout [Carmelo Cortez, 1979].ch8";

fn main() {
    let mut vm = processor::Processor::new();
    vm.load_program(PROGRAM);
    vm.run_program()
}
