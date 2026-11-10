; 42 + 8 * 3 (operações sequenciais)
MOV R0, 42
MOV R1, 8
ADD R0, R1
MOV R2, 3
MUL R0, R2
STORE [0x4000], R0
HLT
