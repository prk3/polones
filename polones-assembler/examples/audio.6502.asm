; nmi handler
;! put address of nmi at prg $7FFA

; reset handler
;! put address of reset at prg $7FFC

; irq/brk handler
;! put address of irq_brk at prg $7FFE


reset
    @check_vblank ; wait until vblank flag is set
    LDA $2002
    AND #%10000000
    BPL @check_vblank

    LDA #%10000000 ; enable nmi
    STA $2000

    LDA #$30
    STA $4000
    LDA #$08
    STA $4001
    LDA #$00
    STA $4002
    STA $4003

    LDA #$30
    STA $4004
    LDA #$08
    STA $4005
    LDA #$00
    STA $4006
    STA $4007

    LDA #$80
    STA $4008
    LDA #$00
    STA $4009
    STA $400A
    STA $400B

    LDA #$30
    STA $400C
    LDA #$00
    STA $400D
    STA $400E
    STA $400F

    STA $4010
    STA $4011
    STA $4012
    STA $4013

    LDA #$0F
    STA $4015
    LDA #$40
    STA $4017

    ; triangle wave
    LDA #$FF
    STA $4008
    LDA #$38
    STA $400A
    LDA #%11111000
    STA $400B

    ; pulse wave
    ; LDA #%10111111
    ; STA $4000
    ; LDA #%00000011
    ; STA $4002
    ; LDA #%00000010
    ; STA $4003

    @sleep_forever
    JMP @sleep_forever


nmi
    RTI


irq_brk
    RTI
