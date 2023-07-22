mod bus;
mod cpu;

fn main() {
    let mut cpu = cpu::CPU::new();
    cpu.run(|_| {});
    println!("Hello, world!");
}
