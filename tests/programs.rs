use blades_cpu_simulator::{assembler::assemble, cpu::Cpu};

fn execute(source: &str) -> Cpu {
    let image = assemble(source).expect("example should assemble");
    let mut cpu = Cpu::new();
    cpu.load(&image.bytes, &image.vectors);
    cpu.run_batch(1_000).expect("example should execute");
    assert!(cpu.halted, "program did not halt");
    cpu
}

#[test]
fn examples_calculator_fibonacci_factorial_and_ram() {
    let calculator = execute(include_str!("../examples/calculator.asm"));
    assert_eq!(
        u16::from_le_bytes([calculator.memory[0x4000], calculator.memory[0x4001]]),
        150
    );
    let fibonacci = execute(include_str!("../examples/fibonacci.asm"));
    assert_eq!(fibonacci.r[1], 21);
    assert_eq!(fibonacci.r[2], 13);
    let factorial = execute(include_str!("../examples/factorial.asm"));
    assert_eq!(factorial.r[1], 120);
    let memory = execute(include_str!("../examples/memory.asm"));
    assert_eq!(memory.r[1], 0xbeef);
}

#[test]
fn examples_hello_subroutine_and_interrupt() {
    let hello = execute(include_str!("../examples/hello.asm"));
    assert_eq!(hello.r[3], 1);
    let routine = execute(include_str!("../examples/subroutines.asm"));
    assert_eq!(routine.r[0], 10);
    assert_eq!(routine.sp, 0xfffe);
    let interrupt = execute(include_str!("../examples/interrupts.asm"));
    assert_eq!(interrupt.r[0], 7);
    assert_eq!(
        u16::from_le_bytes([interrupt.memory[0x4000], interrupt.memory[0x4001]]),
        0xcafe
    );
    assert_eq!(interrupt.sp, 0xfffe);
}
