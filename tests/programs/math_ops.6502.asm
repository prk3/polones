main:
    LDA #100
    STA 200
    lDA 200
    CLC
    ADC #1
    STA 201
    CLC
    SBC #1
    STA 202
    ASL A
    STA 203
    JMP *
