; This ROM tests rendering of sprites over-optimistically copied to secondary
; OAM during sprite ealuation, as described at https://www.nesdev.org/wiki/PPU_sprite_evaluation.
; mason, fceux, and polones don't show these broken sprites.

; nmi handler
;! put address of nmi at prg $7FFA

; reset handler
;! put address of reset at prg $7FFC

; irq/brk handler
;! put address of irq_brk at prg $7FFE

;! put image ./right_line_chr.ppm at chr $0000

reset
    JSR wait_for_vblank

    LDA #%10001000 ; enable nmi
    STA $2000
    LDA #%00011110 ; show sprites and background
    STA $2001

    JSR wait_for_vblank

    JSR set_up_palette

    JSR wait_for_vblank

    JSR prepare_sprite_page

    JSR wait_for_vblank

    ; one sprite at bottom-right, one blue pixel visible
    LDA #$EE
    STA $0300
    LDA #$FF
    STA $0301
    LDA #$FF
    STA $0302
    LDA #$FF
    STA $0303

    JSR wait_for_vblank

    ; copy page 03 to oam
    LDA #$03
    STA $4014

    @sleep_forever
    JMP @sleep_forever


nmi
    RTI


irq_brk
    RTI


wait_for_vblank
    @check_vblank ; wait until vblank flag is set
    LDA $2002
    AND #%10001000
    BPL @check_vblank
    RTS


prepare_sprite_page
    LDX #$00

    @set_sprite
    LDA #$FF
    STA $0300,X
    DEX

    LDA #$FF
    STA $0300,X
    DEX

    LDA #$FF
    STA $0300,X
    DEX

    LDA #$FF
    STA $0300,X
    DEX

    TXA
    BNE @set_sprite

    RTS


set_up_palette
    LDA #$3F ; point PPU address at background color byte
    STA $2006
    LDA #$00
    STA $2006

    LDA #$1D ; write background color
    STA $2007
    LDA #$16 ; write background color
    STA $2007
    LDA #$26 ; write background color
    STA $2007
    LDA #$36 ; write background color
    STA $2007

    LDA #$1D ; write background color
    STA $2007
    LDA #$16 ; write background color
    STA $2007
    LDA #$26 ; write background color
    STA $2007
    LDA #$36 ; write background color
    STA $2007

    LDA #$1D ; write background color
    STA $2007
    LDA #$16 ; write background color
    STA $2007
    LDA #$26 ; write background color
    STA $2007
    LDA #$36 ; write background color
    STA $2007

    LDA #$1D ; write backgrofund color
    STA $2007
    LDA #$16 ; write background color
    STA $2007
    LDA #$26 ; write background color
    STA $2007
    LDA #$36 ; write background color
    STA $2007

    LDA #$1D ; write foreground color
    STA $2007
    LDA #$11 ; write foreground color
    STA $2007
    LDA #$21 ; write foreground color
    STA $2007
    LDA #$31 ; write foreground color
    STA $2007

    LDA #$1D ; write foreground color
    STA $2007
    LDA #$11 ; write foreground color
    STA $2007
    LDA #$21 ; write foreground color
    STA $2007
    LDA #$31 ; write background color
    STA $2007

    LDA #$1D ; write foreground color
    STA $2007
    LDA #$11 ; write foreground color
    STA $2007
    LDA #$21 ; write foreground color
    STA $2007
    LDA #$31 ; write foreground color
    STA $2007

    LDA #$1D ; write foreground color
    STA $2007
    LDA #$11 ; write foreground color
    STA $2007
    LDA #$21 ; write foreground color
    STA $2007
    LDA #$31 ; write foreground color
    STA $2007

    LDA #0 ; zero PPU address to make PPU use the background color we've set
    STA $2006
    STA $2006
    RTS
