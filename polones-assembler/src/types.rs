
#[repr(u8)]
pub enum AddressingMode {
    Accumulator,
    Absolute,
    AbsoluteXIndexed,
    AbsoluteYIndexed,
    Immediate,
    Implied,
    Indirect,
    XIndexedIndirect,
    IndirectYIndexed,
    Relative,
    Zeropage,
    ZeropageXIndexed,
    ZeropageYIndexed,
}

#[derive(PartialEq, Debug)]
pub enum Instruction {
    Opcode(u8),
    OpcodeAndByte(u8, u8),
    OpcodeAndTwoBytes(u8, u8, u8),
    OpcodeAndLabel(u8, String),
    OpcodeAndLabelLocal(u8, String),
}

#[derive(PartialEq, Debug)]
pub enum Directive {
    PutAddressOfSubroutineAtPrgAddress(String, usize),
    PutAddressAtPrgAddress((u8, u8), usize),
    Other(Vec<String>),
}

#[derive(PartialEq, Debug)]
pub enum Item {
    Directive(Directive),
    Instruction(Instruction),
    Label(String),
    LabelLocal(String),
}
