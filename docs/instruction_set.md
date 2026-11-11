# BLADE-16 instruction set (v0.1)

Each instruction begins with one opcode byte. Register identifiers are one byte (low three bits used); immediate words and absolute addresses are little-endian. Every successfully retired instruction increments `cycles` by one. Conditional branches use absolute 16-bit addresses.

| Instruction | Encoding after opcode | Behavior |
|---|---|---|
| NOP, HLT | — | no operation; stop execution |
| MOV Rd, imm16 | reg, word | load immediate; update Z/N |
| MOV Rd, Rs | reg, reg | copy; update Z/N |
| LOAD Rd, [addr] | reg, word | absolute read of a little-endian word; update Z/N |
| LOAD Rd, [Ra] (`0x16`) | reg, reg | read a little-endian word at the address held in Ra; update Z/N |
| STORE [addr], Rs | reg, word | absolute write of a little-endian word |
| STORE [Ra], Rs (`0x17`) | reg, reg | write a little-endian word at the address held in Ra |
| PUSH Rs / POP Rd | reg | word stack transfer |
| ADD/SUB/MUL/DIV/AND/OR/XOR Rd, Rs | reg, reg | result replaces Rd |
| NOT/INC/DEC Rd | reg | unary operation |
| SHL/SHR Rd, count | reg, byte | shift count masked to four bits |
| CMP Ra, Rb | reg, reg | compare without writing a register |
| JMP/JZ/JNZ/JC/JNC/CALL addr | word | unconditional/flag branch; CALL pushes return address |
| RET | — | pop PC |

INT takes a 16-bit vector number (0–255). Its target is the little-endian word in the vector table at `number * 2`; `.VECTOR number, label` initializes an entry. INT saves flags, R0–R7, and return PC on the stack. IRET restores the full context; executing IRET outside an interrupt is an error. DIV is unsigned; division by zero is a reported execution error. Arithmetic is modulo 2^16; C reports carry for ADD/MUL and borrow for SUB/CMP (`a < b`). INC/DEC set C/V when unsigned/signed wrap occurs. Logic and shifts clear V; logic clears C, and shifts report the shifted-out bit. MOV/LOAD leave C/V unchanged. V is signed overflow for ADD/SUB. N is bit 15. Z is equality to zero.
