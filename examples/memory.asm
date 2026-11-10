; Escreve e lê uma palavra em RAM
MOV R0, 0xBEEF
STORE [0x4000], R0
LOAD R1, [0x4000]
HLT
