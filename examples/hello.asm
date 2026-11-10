; BLADE-16 demonstration: arithmetic, compare and conditional branch
MOV R0, 10
MOV R1, 20
ADD R0, R1
MOV R2, 30
CMP R0, R2
JZ SUCCESS
MOV R3, 0
HLT
SUCCESS:
MOV R3, 1
HLT
