# Assembly reference

```asm
; comments begin with semicolon
MOV R0, 0x10
MOV R1, 2
ADD R0, R1
STORE [0x4000], R0
MOV R1, 0x4000
LOAD R2, [R1]      ; register-indirect word read
STORE [R1], R2     ; register-indirect word write
JZ done
done: HLT
```

Interrupt handlers can be registered with `.VECTOR number, label`; `INT number` pushes PC and flags before jumping to the handler, and `IRET` restores both.

Registers are R0 through R7, case-insensitive. `LOAD Rd, [addr]` and `STORE [addr], Rs` use absolute addresses; `LOAD Rd, [Ra]` and `STORE [Ra], Rs` use the 16-bit address stored in a register. Both forms transfer a little-endian word. Numeric literals accept decimal, `0x` hexadecimal or `h`-suffixed hexadecimal notation; a leading minus sign is accepted where the operand's encoding permits signed values. Word operands accept -32768 through 65535 and are encoded modulo 2^16. `.BYTE` accepts -128 through 255 and stores the low byte. Shift counts are encoded in one byte and the CPU uses the low four bits (0–15).

Labels are case-insensitive and may share a line with an instruction. Label names begin with an ASCII letter or underscore and continue with letters, digits or underscores. Directives are `.ORG address`, `.BYTE value[, value...]`, `.WORD value[, value...]` and `.VECTOR number, label-or-address`. `.ORG` can only move forward within the 64 KiB address space; emitted program bytes begin at `0x0200`, with zero-filled gaps. The assembler reports source lines for syntax and operand errors and rejects duplicate labels and interrupt vectors.

An instruction that fails at runtime (for example, division by zero, an invalid opcode or an invalid `IRET`) reports an error without retiring a cycle. The PC is restored to the failing instruction so a debugger can inspect the fault location.
