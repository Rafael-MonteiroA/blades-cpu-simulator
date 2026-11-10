# Assembly reference

```asm
; comments begin with semicolon
MOV R0, 0x10
MOV R1, 2
ADD R0, R1
STORE [0x4000], R0
JZ done
done: HLT
```

Interrupt handlers can be registered with `.VECTOR number, label`; `INT number` pushes PC and flags before jumping to the handler, and `IRET` restores both.

Registers are R0 through R7, case-insensitive. Numeric literals are unsigned decimal, `0x` hexadecimal or `h`-suffixed hexadecimal. Labels are case-insensitive and may share a line with an instruction. Directives: `.ORG address`, `.BYTE value[, value...]`, `.WORD value[, value...]`. Current assembler reports the source line for syntax and operand errors.
