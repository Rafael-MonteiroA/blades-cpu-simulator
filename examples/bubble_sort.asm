; Bubble sort ascending over six 16-bit words.
; Open the program image at 0x0200 to inspect ARRAY after execution.

    MOV R0, 5          ; five passes for six values
OUTER:
    MOV R1, ARRAY      ; address of current pair
    MOV R2, 5          ; five adjacent comparisons per pass
INNER:
    LOAD R3, [R1]      ; left value
    MOV R4, R1
    INC R4
    INC R4             ; address of the next word
    LOAD R5, [R4]      ; right value
    CMP R3, R5
    JC NEXT_PAIR       ; left < right: already ordered
    STORE [R1], R5
    STORE [R4], R3     ; swap the pair
NEXT_PAIR:
    MOV R1, R4
    DEC R2
    JNZ INNER
    DEC R0
    JNZ OUTER
    HLT

ARRAY: .WORD 9, 2, 25, 2, 7, 1
