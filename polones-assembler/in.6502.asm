
; these are some ideas for future directives
;! set mapper 0
;! set submapper 1
;! put address $0123 at prg $1012
;! put sprites abc.png at chr $0123
;! put code at prg $1200
;! set bank_area one prg $0000 $FFFC
;! jump_height .equ #10

; nmi handler
;! put address of nmi at prg $7FFA

; reset handler
;! put address of reset at prg $7FFC

; irq/brk handler
;! put address of irq_brk at prg $7FFE

reset
    LDA #%10000000 ; enable nmi
    STA $0020

    JMP do_nothing

nmi
    LDA 1
    STA $0000
    RTI

irq_brk
    RTI

do_nothing
    JMP do_nothing
