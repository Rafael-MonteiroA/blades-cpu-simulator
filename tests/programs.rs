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

#[test]
fn example_bubble_sort_orders_words_in_ram() {
    let program =
        assemble(include_str!("../examples/bubble_sort.asm")).expect("bubble sort should assemble");
    let array = program.labels["ARRAY"];
    assert_eq!(array, 0x0233);
    let mut cpu = Cpu::new();
    cpu.load(&program.bytes, &program.vectors);
    cpu.run_batch(2_000).expect("bubble sort should execute");

    assert!(cpu.halted, "bubble sort did not halt");
    let sorted: Vec<u16> = (0..6)
        .map(|index| {
            let address = array.wrapping_add(index * 2) as usize;
            u16::from_le_bytes([cpu.memory[address], cpu.memory[address + 1]])
        })
        .collect();
    assert_eq!(sorted, [1, 2, 2, 7, 9, 25]);
    assert!(cpu.cycles < 2_000);
}
