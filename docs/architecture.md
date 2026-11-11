# BLADE-16 architecture

BLADE-16 is a 16-bit accumulator-free virtual processor with eight general registers R0–R7, a 16-bit PC, SP, IR, and Z/C/N/V status flags. Addressing spans 0x0000–0xFFFF (64 KiB). RAM bytes and encoded words use little-endian ordering. The assembler loads the image at 0x0200 unless `.ORG` selects a later address.

The engine performs one instruction synchronously per `step`: fetch opcode and operands, decode opcode, execute, commit state, and increment the cycle counter. `run_batch` applies a caller-selected instruction budget and checks breakpoints before each instruction. UI run uses a bounded batch per egui frame.

LOAD and STORE support both absolute word addresses and register-indirect addresses. In the indirect form, a general register supplies the 16-bit RAM address, allowing Assembly programs to walk arrays and buffers.

The stack is in RAM, grows toward lower addresses, and stores 16-bit words. SP starts at 0xFFFE. CALL pushes the return PC and RET pops it. INT pushes flags, R0–R7, and return PC; IRET restores them. The ALU is represented by the arithmetic/logical operations in the decode/execute engine. Z and N are updated by result-producing ALU and MOV/LOAD instructions; C and V follow arithmetic carry/borrow and signed two's-complement overflow for ADD/SUB. Logic ops clear C/V and set Z/N; shifts set C from the shifted-out bits. Division is unsigned and division by zero returns a CPU error without advancing cycle count.

Memory accesses wrap at the 16-bit address boundary. The UI highlights register and memory writes made by the last successful instruction. No external thread or nondeterministic device is involved.
