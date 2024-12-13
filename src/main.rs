mod drivers;
mod processor;

fn main() {
    // TODO: Handle malformed arguments
    let args: Vec<String> = std::env::args().collect();

    let mut vm = processor::Processor::new();
    vm.load_program(&args[1]);
    vm.run_program()
}
