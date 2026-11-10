; Handler modifica RAM; INT preserva os registradores
.VECTOR 1, HANDLER
MOV R0, 7
MOV R3, 1
INT 1
HLT
HANDLER:
MOV R2, 0xCAFE
STORE [0x4000], R2
IRET
