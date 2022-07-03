; This program shows all colors a NES can display.
; The background color changes every 30 frames (0.5 second).

; nmi handler
;! put address of nmi at prg $7FFA

; reset handler
;! put address of reset at prg $7FFC

; irq/brk handler
;! put address of irq_brk at prg $7FFE


reset
    LDA #0 ; clear memory we'll be using
    STA $00 ; frame count
    STA $01 ; background color

    @check_vblank ; wait until vblank flag is set
    LDA $2002
    AND #%10000000
    BPL @check_vblank

    LDA #%10000000 ; enable nmi
    STA $2000

    JSR set_background_color
    JSR update_background_color

    @sleep_forever
    JMP @sleep_forever


nmi
    LDA $00 ; check if frame count is 29
    CMP #29
    BNE @increment_frame_counter

    JSR set_background_color
    JSR update_background_color

    LDA #0 ; reset frame counter
    STA $00
    RTI

    @increment_frame_counter
    INC $00
    RTI


irq_brk
    RTI


set_background_color
    LDA #$3F ; point PPU address at background color byte
    STA $2006
    LDA #$00
    STA $2006

    LDA $01 ; write background color
    STA $2007

    LDA #0 ; zero PPU address to make PPU use the background color we've set
    STA $2006
    STA $2006
    RTS


update_background_color
    LDA $01
    CLC
    ADC #1
    AND #%00111111
    STA $01
    RTS
