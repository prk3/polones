use std::marker::PhantomData;

use super::bus::Bus;

pub struct Cpu<B> {
    // registers
    accumulator: u8,
    x_index: u8,
    y_index: u8,
    program_counter: u16,
    stack_pointer: u8,
    status_register: StatusRegister,

    // other
    opcode: u8, // the opcode that is currently being handled
    sleep_cycles: u8, // how many cycles the current instruction should take
    operand_address: u16, // address of operand (not used when address mode is accumulator of implied)
    operand_accumulator: bool, // whether the operand is accumulator
    crossed_page_boundary: bool, // whether sleep cycles might be increased by crossing the page boundary
    program_counter_offset: i8, // relative jump target (for branch instructions)

    // for generic bus
    _bus: PhantomData<B>,
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
impl<B: Bus> Cpu<B> {
    pub fn new() -> Self {
        Self {
            accumulator: 0,
            x_index: 0,
            y_index: 0,
            program_counter: 0,
            stack_pointer: 0,
            status_register: StatusRegister::new(),

            opcode: 0,
            sleep_cycles: 0,
            operand_address: 0,
            operand_accumulator: false,
            crossed_page_boundary: false,
            program_counter_offset: 0,

            _bus: PhantomData::default(),
        }
    }
    // thanks
    // https://masswerk.at/6502/6502_instruction_set.html

    const INSTRUCTION_SET: [fn (cpu: &mut Self, bus: &B); 256] = [
        /* HI   LO    0             1             2             3             4             5             6             7             8             9             A             B             C             D             E             F    */
        /* 0 */ Self::run_00, Self::run_01, Self::run_xx, Self::run_xx, Self::run_xx, Self::run_05, Self::run_06, Self::run_xx, Self::run_08, Self::run_09, Self::run_0A, Self::run_xx, Self::run_xx, Self::run_0D, Self::run_0E, Self::run_xx,
        /* 1 */ Self::run_10, Self::run_11, Self::run_xx, Self::run_xx, Self::run_xx, Self::run_15, Self::run_16, Self::run_xx, Self::run_18, Self::run_19, Self::run_xx, Self::run_xx, Self::run_xx, Self::run_1D, Self::run_1E, Self::run_xx,
        /* 2 */ Self::run_20, Self::run_21, Self::run_xx, Self::run_xx, Self::run_24, Self::run_25, Self::run_26, Self::run_xx, Self::run_28, Self::run_29, Self::run_2A, Self::run_xx, Self::run_2C, Self::run_2D, Self::run_2E, Self::run_xx,
        /* 3 */ Self::run_30, Self::run_31, Self::run_xx, Self::run_xx, Self::run_xx, Self::run_35, Self::run_36, Self::run_xx, Self::run_38, Self::run_39, Self::run_xx, Self::run_xx, Self::run_xx, Self::run_3D, Self::run_3E, Self::run_xx,
        /* 4 */ Self::run_40, Self::run_41, Self::run_xx, Self::run_xx, Self::run_xx, Self::run_45, Self::run_46, Self::run_xx, Self::run_48, Self::run_49, Self::run_4A, Self::run_xx, Self::run_4C, Self::run_4D, Self::run_4E, Self::run_xx,
        /* 5 */ Self::run_50, Self::run_51, Self::run_xx, Self::run_xx, Self::run_xx, Self::run_55, Self::run_56, Self::run_xx, Self::run_58, Self::run_59, Self::run_xx, Self::run_xx, Self::run_xx, Self::run_5D, Self::run_5E, Self::run_xx,
        /* 6 */ Self::run_60, Self::run_61, Self::run_xx, Self::run_xx, Self::run_xx, Self::run_65, Self::run_66, Self::run_xx, Self::run_68, Self::run_69, Self::run_6A, Self::run_xx, Self::run_6C, Self::run_6D, Self::run_6E, Self::run_xx,
        /* 7 */ Self::run_70, Self::run_71, Self::run_xx, Self::run_xx, Self::run_xx, Self::run_75, Self::run_76, Self::run_xx, Self::run_78, Self::run_79, Self::run_xx, Self::run_xx, Self::run_xx, Self::run_7D, Self::run_7E, Self::run_xx,
        /* 8 */ Self::run_xx, Self::run_81, Self::run_xx, Self::run_xx, Self::run_84, Self::run_85, Self::run_86, Self::run_xx, Self::run_88, Self::run_xx, Self::run_8A, Self::run_xx, Self::run_8C, Self::run_8D, Self::run_8E, Self::run_xx,
        /* 9 */ Self::run_90, Self::run_91, Self::run_xx, Self::run_xx, Self::run_94, Self::run_95, Self::run_96, Self::run_xx, Self::run_98, Self::run_99, Self::run_9A, Self::run_xx, Self::run_xx, Self::run_9D, Self::run_xx, Self::run_xx,
        /* A */ Self::run_A0, Self::run_A1, Self::run_A2, Self::run_xx, Self::run_A4, Self::run_A5, Self::run_A6, Self::run_xx, Self::run_A8, Self::run_A9, Self::run_AA, Self::run_xx, Self::run_AC, Self::run_AD, Self::run_AE, Self::run_xx,
        /* B */ Self::run_B0, Self::run_B1, Self::run_xx, Self::run_xx, Self::run_B4, Self::run_B5, Self::run_B6, Self::run_xx, Self::run_B8, Self::run_B9, Self::run_BA, Self::run_xx, Self::run_BC, Self::run_BD, Self::run_BE, Self::run_xx,
        /* C */ Self::run_C0, Self::run_C1, Self::run_xx, Self::run_xx, Self::run_C4, Self::run_C5, Self::run_C6, Self::run_xx, Self::run_C8, Self::run_C9, Self::run_CA, Self::run_xx, Self::run_CC, Self::run_CD, Self::run_CE, Self::run_xx,
        /* D */ Self::run_D0, Self::run_D1, Self::run_xx, Self::run_xx, Self::run_xx, Self::run_D5, Self::run_D6, Self::run_xx, Self::run_D8, Self::run_D9, Self::run_xx, Self::run_xx, Self::run_xx, Self::run_DD, Self::run_DE, Self::run_xx,
        /* E */ Self::run_E0, Self::run_E1, Self::run_xx, Self::run_xx, Self::run_E4, Self::run_E5, Self::run_E6, Self::run_xx, Self::run_E8, Self::run_E9, Self::run_EA, Self::run_xx, Self::run_EC, Self::run_ED, Self::run_EE, Self::run_xx,
        /* F */ Self::run_F0, Self::run_F1, Self::run_xx, Self::run_xx, Self::run_xx, Self::run_F5, Self::run_F6, Self::run_xx, Self::run_F8, Self::run_F9, Self::run_xx, Self::run_xx, Self::run_xx, Self::run_FD, Self::run_FE, Self::run_xx,
    ];

    pub fn reset(&mut self, bus: &B) {
        let low = bus.read(0xFFFC);
        let high = bus.read(0xFFFC + 1);
        self.program_counter = ((high as u16) << 8) | low as u16;
        self.sleep_cycles = 8;
    }

    pub fn tick(&mut self, bus: &B) {
        if self.sleep_cycles > 0 {
            self.sleep_cycles -= 1;
            return
        }

        self.opcode = bus.read(self.program_counter);
        self.sleep_cycles = 0;
        self.operand_address = 0;
        self.operand_accumulator = false;
        self.program_counter_offset = 0;
        self.crossed_page_boundary = false;

        Self::INSTRUCTION_SET[self.opcode as usize](self, bus);
    }

    pub fn run_one_instruction(&mut self, bus: &B) {
        self.sleep_cycles = 0;
        self.tick(bus);
        self.sleep_cycles = 0;
    }

    fn get_operand_byte(&self, bus: &B) -> u8 {
        if self.operand_accumulator {
            self.accumulator
        } else {
            bus.read(self.operand_address)
        }
    }

    fn set_operand_byte(&mut self, bus: &B, byte: u8) {
        if self.operand_accumulator {
            self.accumulator = byte;
        } else {
            bus.write(self.operand_address, byte)
        }
    }

    // Helper function for all kind of branch instruction.
    // Branches to pc + program_counter_offset.
    // Adds 1 or 2 sleep cycles, depending on the jump address.
    fn branch(&mut self, bus: &B) {
        let old_program_counter = self.program_counter;

        self.program_counter = u16_address_offset(self.program_counter, self.program_counter_offset);

        if self.program_counter & 0xFF00 == old_program_counter & 0xFF00 {
            self.sleep_cycles += 1;
        } else {
            self.sleep_cycles += 2;
        }
    }

    // address modes

    fn address_mode_absolute(&mut self, bus: &B) {
        let low = bus.read(self.program_counter.wrapping_add(1));
        let high = bus.read(self.program_counter.wrapping_add(2));

        self.operand_address = ((high as u16) << 8) | (low as u16);

        self.program_counter = self.program_counter.wrapping_add(3);
    }

    fn address_mode_absolute_x_indexed(&mut self, bus: &B) {
        let low = bus.read(self.program_counter.wrapping_add(1));
        let high = bus.read(self.program_counter.wrapping_add(2));

        self.operand_address = ((high as u16) << 8) | (low as u16).wrapping_add(self.x_index as u16);

        if (self.operand_address >> 8) as u8 != high {
            self.crossed_page_boundary = true;
        }

        self.program_counter = self.program_counter.wrapping_add(3);
    }

    fn address_mode_absolute_y_indexed(&mut self, bus: &B) {
        let low = bus.read(self.program_counter.wrapping_add(1));
        let high = bus.read(self.program_counter.wrapping_add(2));

        self.operand_address = ((high as u16) << 8) | (low as u16).wrapping_add(self.y_index as u16);

        if (self.operand_address >> 8) as u8 != high {
            self.crossed_page_boundary = true;
        }

        self.program_counter = self.program_counter.wrapping_add(3);
    }

    fn address_mode_immediate(&mut self, bus: &B) {
        self.operand_address = self.program_counter.wrapping_add(1);

        self.program_counter = self.program_counter.wrapping_add(2);
    }

    fn address_mode_implied(&mut self, bus: &B) {
        self.program_counter = self.program_counter.wrapping_add(1);
    }

    fn address_mode_indirect(&mut self, bus: &B) {
        let ptr_low = bus.read(self.program_counter.wrapping_add(1));
        let ptr_high = bus.read(self.program_counter.wrapping_add(2));

        let ptr = ((ptr_high as u16) << 8) | ptr_low as u16;

        let low = bus.read(ptr);
        let high = bus.read(ptr.wrapping_add(1));

        self.operand_address = ((high as u16) << 8) | (low as u16);
        self.program_counter = self.program_counter.wrapping_add(3);
    }

    fn address_mode_x_indexed_indirect(&mut self, bus: &B) {
        let ptr_low = bus.read(self.program_counter.wrapping_add(1)).wrapping_add(self.x_index);

        let ptr = ptr_low as u16;

        let low = bus.read(ptr);
        let high = bus.read((ptr + 1) & 0x00FF);

        self.operand_address = ((high as u16) << 8) | (low as u16);
        self.program_counter = self.program_counter.wrapping_add(2);
    }

    fn address_mode_indirect_y_indexed(&mut self, bus: &B) {
        let ptr_low = bus.read(self.program_counter.wrapping_add(1));

        let ptr = ptr_low as u16;

        let low = bus.read(ptr);
        let high = bus.read((ptr + 1) & 0x00FF);

        self.operand_address = (((high as u16) << 8) | (low as u16)).wrapping_add(self.x_index as u16);

        if (self.operand_address >> 8) as u8 != high {
            self.crossed_page_boundary = true;
        }

        self.program_counter = self.program_counter.wrapping_add(2);
    }

    fn address_mode_relative(&mut self, bus: &B) {
        self.program_counter_offset = u8_to_i8(bus.read(self.program_counter.wrapping_add(1)));

        self.program_counter = self.program_counter.wrapping_add(2);
    }

    fn address_mode_zeropage(&mut self, bus: &B) {
        let low = bus.read(self.program_counter.wrapping_add(1));

        self.operand_address = low as u16;

        self.program_counter = self.program_counter.wrapping_add(2);
    }

    fn address_mode_zeropage_x_indexed(&mut self, bus: &B) {
        let low = bus.read(self.program_counter.wrapping_add(1));

        self.operand_address = low.wrapping_add(self.x_index) as u16;

        self.program_counter = self.program_counter.wrapping_add(2);
    }

    fn address_mode_zeropage_y_indexed(&mut self, bus: &B) {
        let low = bus.read(self.program_counter.wrapping_add(1));

        self.operand_address = low.wrapping_add(self.y_index) as u16;

        self.program_counter = self.program_counter.wrapping_add(2);
    }

    // operations

    // ADC
    fn add_with_carry(&mut self, bus: &B) {
        let operand_byte = self.get_operand_byte(bus);
        let old_accumulator = self.accumulator;

        let sum = (self.accumulator as u16).wrapping_add(operand_byte as u16).wrapping_add(self.status_register.get_carry() as u16);
        self.accumulator = sum as u8;

        self.status_register.set_negative(self.accumulator & 0b10000000 > 0);
        self.status_register.set_zero(self.accumulator == 0);
        self.status_register.set_carry(sum & 0xFF > 0);
        self.status_register.set_overflow((old_accumulator ^ self.accumulator) & (old_accumulator ^ operand_byte) & 0b10000000 > 1);
    }

    // AND
    fn and(&mut self, bus: &B) {
        let operand_byte = self.get_operand_byte(bus);
        let old_accumulator = self.accumulator;

        self.accumulator = old_accumulator.wrapping_add(operand_byte).wrapping_add(self.status_register.get_carry() as u8);

        self.status_register.set_negative(self.accumulator & 0b10000000 > 0);
        self.status_register.set_zero(self.accumulator == 0);
    }

    // ASL
    fn arithmetic_shift_left(&mut self, bus: &B) {
        let operand_byte = self.get_operand_byte(bus);
        let carry = operand_byte & 0b10000000;

        self.accumulator = (self.accumulator & 0b01111111) << 1;

        self.status_register.set_zero(self.accumulator == 0);
        self.status_register.set_negative(self.accumulator & 0b10000000 > 0);
        self.status_register.set_carry(carry > 0);
    }

    // BCC
    fn branch_on_carry_clear(&mut self, bus: &B) {
        if !self.status_register.get_carry() {
            self.branch(bus);
        }
    }

    // BCS
    fn branch_on_carry_set(&mut self, bus: &B) {
        if self.status_register.get_carry() {
            self.branch(bus);
        }
    }

    // BCS
    fn branch_on_equal(&mut self, bus: &B) {
        if self.status_register.get_zero() {
            self.branch(bus);
        }
    }

    // BCS
    fn bit_test(&mut self, bus: &B) {
        let operand_byte = self.get_operand_byte(bus);
        let accumulator_and_operand = self.accumulator & operand_byte;

        self.status_register.set_negative(operand_byte & 0b10000000 > 0);
        self.status_register.set_overflow(operand_byte & 0b01000000 > 0);
        self.status_register.set_zero(accumulator_and_operand == 0);
    }

    // BMI
    fn branch_on_minus(&mut self, bus: &B) {
        if self.status_register.get_negative() {
            self.branch(bus);
        }
    }

    // BNE
    fn branch_on_not_equal(&mut self, bus: &B) {
        if !self.status_register.get_zero() {
            self.branch(bus);
        }
    }

    // BPL
    fn branch_on_plus(&mut self, bus: &B) {
        if !self.status_register.get_negative() {
            self.branch(bus);
        }
    }

    // BRK
    fn break_or_interrupt(&mut self, bus: &B) {
        self.stack_pointer = self.stack_pointer.wrapping_sub(2);
        let high = (self.program_counter.wrapping_add(2) >> 8) as u8;
        let low = self.program_counter.wrapping_add(2) as u8;
        bus.write(0x0100 + self.stack_pointer as u16, high);
        bus.write(0x0100 + self.stack_pointer.wrapping_add(1) as u16, low);

        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
        let mut status_register = self.status_register;
        status_register.set_ignored(true);
        status_register.set_break(true);
    }

    // BVC
    fn branch_on_overflow_clear(&mut self, bus: &B) {
        if !self.status_register.get_overflow() {
            self.branch(bus);
        }
    }

    // BVS
    fn branch_on_overflow_set(&mut self, bus: &B) {
        if self.status_register.get_overflow() {
            self.branch(bus);
        }
    }

    // CLC
    fn clear_carry(&mut self, bus: &B) {
        self.status_register.set_carry(false);
    }

    // CLD
    fn clear_decimal(&mut self, bus: &B) {
        self.status_register.set_decimal(false);
    }

    // CLI
    fn clear_interrupt_disable(&mut self, bus: &B) {
        self.status_register.set_interrupt(false);
    }

    // CLV
    fn clear_overflow(&mut self, bus: &B) {
        self.status_register.set_overflow(false);
    }

    // CMP
    fn compare(&mut self, bus: &B) {
        let operand_byte = self.get_operand_byte(bus);
        let accumulator_minus_operand = self.accumulator.wrapping_sub(operand_byte);
        self.status_register.set_negative(accumulator_minus_operand & 0b10000000 > 0);
        self.status_register.set_zero(accumulator_minus_operand == 0);
        self.status_register.set_carry(accumulator_minus_operand > self.accumulator);
    }

    // CPX
    fn compare_with_x(&mut self, bus: &B) {
        let operand_byte = self.get_operand_byte(bus);
        let x_minus_operand = self.x_index.wrapping_sub(operand_byte);
        self.status_register.set_negative(x_minus_operand & 0b10000000 > 0);
        self.status_register.set_zero(x_minus_operand == 0);
        self.status_register.set_carry(x_minus_operand > self.x_index);
    }

    // CPY
    fn compare_with_y(&mut self, bus: &B) {
        let operand_byte = self.get_operand_byte(bus);
        let y_minus_operand = self.x_index.wrapping_sub(operand_byte);
        self.status_register.set_negative(y_minus_operand & 0b10000000 > 0);
        self.status_register.set_zero(y_minus_operand == 0);
        self.status_register.set_carry(y_minus_operand > self.y_index);
    }

    // DEC
    fn decrement(&mut self, bus: &B) {
        let operand_byte = self.get_operand_byte(bus).wrapping_sub(1);
        self.status_register.set_negative(operand_byte & 0b10000000 > 0);
        self.status_register.set_zero(operand_byte == 0);
        self.set_operand_byte(bus, operand_byte);
    }

    // DEX
    fn decrement_x(&mut self, bus: &B) {
        self.x_index = self.x_index.wrapping_sub(1);
        self.status_register.set_negative(self.x_index & 0b10000000 > 0);
        self.status_register.set_zero(self.x_index == 0);
    }

    // DEY
    fn decrement_y(&mut self, bus: &B) {
        self.y_index = self.y_index.wrapping_sub(1);
        self.status_register.set_negative(self.y_index & 0b10000000 > 0);
        self.status_register.set_zero(self.y_index == 0);
    }

    // EOR
    fn exclusive_or(&mut self, bus: &B) {
        let operand_byte = self.get_operand_byte(bus);
        self.accumulator = self.accumulator ^ operand_byte;
        self.status_register.set_negative(self.accumulator & 0b10000000 > 0);
        self.status_register.set_zero(self.accumulator == 0);
    }

    // INC
    fn increment(&mut self, bus: &B) {
        let operand_byte = self.get_operand_byte(bus).wrapping_add(1);
        self.status_register.set_negative(operand_byte & 0b10000000 > 0);
        self.status_register.set_zero(operand_byte == 0);
        self.set_operand_byte(bus, operand_byte);
    }

    // INX
    fn increment_x(&mut self, bus: &B) {
        self.x_index = self.x_index.wrapping_add(1);
        self.status_register.set_negative(self.x_index & 0b10000000 > 0);
        self.status_register.set_zero(self.x_index == 0);
    }

    // INY
    fn increment_y(&mut self, bus: &B) {
        self.y_index = self.y_index.wrapping_add(1);
        self.status_register.set_negative(self.y_index & 0b10000000 > 0);
        self.status_register.set_zero(self.y_index == 0);
    }

    // JMP
    fn jump(&mut self, bus: &B) {
        self.program_counter = self.operand_address;
    }

    // JSR
    fn jump_subroutine(&mut self, bus: &B) {
        let program_counter_to_save = self.program_counter.wrapping_sub(1);
        self.stack_pointer = self.stack_pointer.wrapping_sub(2);
        bus.write(0x0100 + self.stack_pointer as u16, (program_counter_to_save >> 8) as u8);
        bus.write(0x0100 + self.stack_pointer.wrapping_add(1) as u16, program_counter_to_save as u8);
        self.program_counter = self.operand_address;
    }

    // LDA
    fn load_accumulator(&mut self, bus: &B) {
        self.accumulator = self.get_operand_byte(bus);
        self.status_register.set_negative(self.accumulator & 0b10000000 > 0);
        self.status_register.set_zero(self.accumulator == 0);
    }

    // LDX
    fn load_x(&mut self, bus: &B) {
        self.x_index = self.get_operand_byte(bus);
        self.status_register.set_negative(self.x_index & 0b10000000 > 0);
        self.status_register.set_zero(self.x_index == 0);
    }

    // LDY
    fn load_y(&mut self, bus: &B) {
        self.y_index = self.get_operand_byte(bus);
        self.status_register.set_negative(self.y_index & 0b10000000 > 0);
        self.status_register.set_zero(self.y_index == 0);
    }

    // LSR
    fn logical_shift_right(&mut self, bus: &B) {
        let mut operand_byte = self.get_operand_byte(bus);
        let leftmost_bit = operand_byte & 1;
        operand_byte = operand_byte >> 1;
        self.status_register.set_negative(false);
        self.status_register.set_zero(operand_byte == 0);
        self.status_register.set_carry(leftmost_bit > 0);
        self.set_operand_byte(bus, operand_byte);
    }

    // NOP
    fn no_operation(&mut self, bus: &B) {}

    // ORA
    fn or_with_accumulator(&mut self, bus: &B) {
        let operand_byte = self.get_operand_byte(bus);
        self.accumulator = self.accumulator | operand_byte;
        self.status_register.set_negative(self.accumulator & 0b10000000 > 0);
        self.status_register.set_zero(self.accumulator == 0);
    }

    // PHA
    fn push_accumulator(&mut self, bus: &B) {
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
        bus.write(0x0100 + self.stack_pointer as u16, self.accumulator);
    }

    // PHP
    fn push_processor_status(&mut self, bus: &B) {
        let mut status_register_copy = self.status_register;
        status_register_copy.set_ignored(true);
        status_register_copy.set_break(true);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
        bus.write(0x0100 + self.stack_pointer as u16, status_register_copy.0);
    }

    // PLA
    fn pull_accumulator(&mut self, bus: &B) {
        self.accumulator = bus.read(0x0100 + self.stack_pointer as u16);
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.status_register.set_negative(self.accumulator & 0b10000000 > 0);
        self.status_register.set_zero(self.accumulator == 0);
    }

    // PLP
    fn pull_processor_status(&mut self, bus: &B) {
        let mut processor_status = StatusRegister(bus.read(0x0100 + self.stack_pointer as u16));
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        processor_status.set_ignored(false);
        processor_status.set_break(false);
        self.status_register = processor_status;
    }

    // ROL
    fn rotate_left(&mut self, bus: &B) {
        let mut operand_byte = self.get_operand_byte(bus);
        let leftmost_bit = operand_byte & 0b10000000;
        operand_byte = (operand_byte << 1) | self.status_register.get_carry() as u8;
        self.status_register.set_carry(leftmost_bit > 0);
        self.status_register.set_negative(operand_byte & 0b10000000 > 0);
        self.status_register.set_zero(operand_byte == 0);
        self.set_operand_byte(bus, operand_byte);
    }

    // ROR
    fn rotate_right(&mut self, bus: &B) {
        let mut operand_byte = self.get_operand_byte(bus);
        let rightmost_bit = operand_byte & 1;
        operand_byte = (operand_byte >> 1) | ((self.status_register.get_carry() as u8) << 7);
        self.status_register.set_carry(rightmost_bit > 0);
        self.status_register.set_negative(operand_byte & 0b10000000 > 0);
        self.status_register.set_zero(operand_byte == 0);
        self.set_operand_byte(bus, operand_byte);
    }

    // RTI
    fn return_from_interrupt(&mut self, bus: &B) {
        self.status_register = StatusRegister(bus.read(self.stack_pointer as u16));
        self.status_register.set_break(false);
        self.status_register.set_ignored(false);
        let high = bus.read(self.stack_pointer.wrapping_add(1) as u16);
        let low = bus.read(self.stack_pointer.wrapping_add(2) as u16);
        self.stack_pointer = self.stack_pointer.wrapping_add(3);
        self.program_counter = ((high as u16) << 8) | (low as u16);
    }

    // RTS
    fn return_from_subroutine(&mut self, bus: &B) {
        let high = bus.read(self.stack_pointer as u16);
        let low = bus.read(self.stack_pointer.wrapping_add(1) as u16);
        self.stack_pointer = self.stack_pointer.wrapping_add(2);
        self.program_counter = (((high as u16) << 8) | (low as u16)).wrapping_add(1);
    }

    // SBC
    fn subtract_with_carry(&mut self, bus: &B) {
        let operand_byte = self.get_operand_byte(bus);
        let old_accumulator = self.accumulator;

        let difference = (self.accumulator as u16).wrapping_sub(operand_byte as u16).wrapping_sub(self.status_register.get_carry() as u16);
        self.accumulator = difference as u8;

        self.status_register.set_negative(self.accumulator & 0b10000000 > 0);
        self.status_register.set_zero(self.accumulator == 0);
        self.status_register.set_carry(difference & 0xFF > 0);
        self.status_register.set_overflow((old_accumulator ^ self.accumulator) & (old_accumulator ^ operand_byte) & 0b10000000 > 1);
    }

    // SEC
    fn set_carry(&mut self, bus: &B) {
        self.status_register.set_carry(true);
    }

    // SED
    fn set_decimal(&mut self, bus: &B) {
        self.status_register.set_decimal(true);
    }

    // SEI
    fn set_interrupt_disable(&mut self, bus: &B) {
        self.status_register.set_interrupt(true);
    }

    // STA
    fn store_accumulator(&mut self, bus: &B) {
        self.set_operand_byte(bus, self.accumulator);
    }

    // STX
    fn store_x(&mut self, bus: &B) {
        self.set_operand_byte(bus, self.x_index);
    }

    // STY
    fn store_y(&mut self, bus: &B) {
        self.set_operand_byte(bus, self.y_index);
    }

    // TAX
    fn transfer_accumulator_to_x(&mut self, bus: &B) {
        self.x_index = self.accumulator;
        self.status_register.set_negative(self.accumulator & 0b10000000 > 0);
        self.status_register.set_zero(self.accumulator == 0);
    }

    // TAY
    fn transfer_accumulator_to_y(&mut self, bus: &B) {
        self.y_index = self.accumulator;
        self.status_register.set_negative(self.accumulator & 0b10000000 > 0);
        self.status_register.set_zero(self.accumulator == 0);
    }

    // TSX
    fn transfer_stack_pointer_to_x(&mut self, bus: &B) {
        self.x_index = self.stack_pointer;
        self.status_register.set_negative(self.accumulator & 0b10000000 > 0);
        self.status_register.set_zero(self.accumulator == 0);
    }

    // TXA
    fn transfer_x_to_accumulator(&mut self, bus: &B) {
        self.accumulator = self.x_index;
        self.status_register.set_negative(self.accumulator & 0b10000000 > 0);
        self.status_register.set_zero(self.accumulator == 0);
    }

    // TXS
    fn transfer_x_to_stack_pointer(&mut self, bus: &B) {
        self.stack_pointer = self.x_index;
    }

    // TYA
    fn transfer_y_to_accumulator(&mut self, bus: &B) {
        self.accumulator = self.y_index;
        self.status_register.set_negative(self.accumulator & 0b10000000 > 0);
        self.status_register.set_zero(self.accumulator == 0);
    }

    // operation codes handlers

    fn run_00(&mut self, bus: &B) {
        self.address_mode_implied(bus);
        self.break_or_interrupt(bus);
        self.sleep_cycles += 7;
    }

    fn run_10(&mut self, bus: &B) {
        self.address_mode_relative(bus);
        self.branch_on_plus(bus);
        self.sleep_cycles += 2;
    }

    fn run_20(&mut self, bus: &B) {
        self.address_mode_absolute(bus);
        self.jump_subroutine(bus);
        self.sleep_cycles += 6;
    }

    fn run_30(&mut self, bus: &B) {
        self.address_mode_relative(bus);
        self.branch_on_minus(bus);
        self.sleep_cycles += 2;
    }

    fn run_40(&mut self, bus: &B) {
        self.address_mode_implied(bus);
        self.return_from_interrupt(bus);
        self.sleep_cycles += 6;
    }

    fn run_50(&mut self, bus: &B) {
        self.address_mode_relative(bus);
        self.branch_on_overflow_clear(bus);
        self.sleep_cycles += 2;
    }

    fn run_60(&mut self, bus: &B) {
        self.address_mode_implied(bus);
        self.return_from_subroutine(bus);
        self.sleep_cycles += 6;
    }

    fn run_70(&mut self, bus: &B) {
        self.address_mode_relative(bus);
        self.branch_on_overflow_set(bus);
        self.sleep_cycles += 2;
    }

    // 80 - illegal

    fn run_90(&mut self, bus: &B) {
        self.address_mode_relative(bus);
        self.branch_on_carry_clear(bus);
        self.sleep_cycles += 2;
    }

    fn run_A0(&mut self, bus: &B) {
        self.address_mode_immediate(bus);
        self.load_y(bus);
        self.sleep_cycles += 2;
    }

    fn run_B0(&mut self, bus: &B) {
        self.address_mode_relative(bus);
        self.branch_on_carry_set(bus);
        self.sleep_cycles += 2;
    }

    fn run_C0(&mut self, bus: &B) {
        self.address_mode_immediate(bus);
        self.compare_with_y(bus);
        self.sleep_cycles += 2;
    }

    fn run_D0(&mut self, bus: &B) {
        self.address_mode_relative(bus);
        self.branch_on_not_equal(bus);
        self.sleep_cycles += 2;
    }

    fn run_E0(&mut self, bus: &B) {
        self.address_mode_immediate(bus);
        self.compare_with_x(bus);
        self.sleep_cycles += 2;
    }

    fn run_F0(&mut self, bus: &B) {
        self.address_mode_relative(bus);
        self.branch_on_equal(bus);
        self.sleep_cycles += 2;
    }

    fn run_01(&mut self, bus: &B) {
        self.address_mode_x_indexed_indirect(bus);
        self.or_with_accumulator(bus);
        self.sleep_cycles += 6;
    }

    fn run_11(&mut self, bus: &B) {
        self.address_mode_indirect_y_indexed(bus);
        self.or_with_accumulator(bus);
        self.sleep_cycles += 5;
        self.sleep_cycles += self.crossed_page_boundary as u8;
    }

    fn run_21(&mut self, bus: &B) {
        self.address_mode_x_indexed_indirect(bus);
        self.and(bus);
        self.sleep_cycles += 6;
    }

    fn run_31(&mut self, bus: &B) {
        self.address_mode_indirect_y_indexed(bus);
        self.and(bus);
        self.sleep_cycles += 5;
        self.sleep_cycles += self.crossed_page_boundary as u8;
    }

    fn run_41(&mut self, bus: &B) {
        self.address_mode_x_indexed_indirect(bus);
        self.exclusive_or(bus);
        self.sleep_cycles += 6;
    }

    fn run_51(&mut self, bus: &B) {
        self.address_mode_indirect_y_indexed(bus);
        self.exclusive_or(bus);
        self.sleep_cycles += 5;
        self.sleep_cycles += self.crossed_page_boundary as u8;
    }

    fn run_61(&mut self, bus: &B) {
        self.address_mode_x_indexed_indirect(bus);
        self.add_with_carry(bus);
        self.sleep_cycles += 6;
    }

    fn run_71(&mut self, bus: &B) {
        self.address_mode_indirect_y_indexed(bus);
        self.add_with_carry(bus);
        self.sleep_cycles += 5;
        self.sleep_cycles += self.crossed_page_boundary as u8;
    }

    fn run_81(&mut self, bus: &B) {
        self.address_mode_x_indexed_indirect(bus);
        self.store_accumulator(bus);
        self.sleep_cycles += 6;
    }

    fn run_91(&mut self, bus: &B) {
        self.address_mode_indirect_y_indexed(bus);
        self.store_accumulator(bus);
        self.sleep_cycles += 6;
    }

    fn run_A1(&mut self, bus: &B) {
        self.address_mode_x_indexed_indirect(bus);
        self.load_accumulator(bus);
        self.sleep_cycles += 6;
    }

    fn run_B1(&mut self, bus: &B) {
        self.address_mode_indirect_y_indexed(bus);
        self.load_accumulator(bus);
        self.sleep_cycles += 5;
        self.sleep_cycles += self.crossed_page_boundary as u8;
    }

    fn run_C1(&mut self, bus: &B) {
        self.address_mode_x_indexed_indirect(bus);
        self.compare(bus);
        self.sleep_cycles += 6;
    }

    fn run_D1(&mut self, bus: &B) {
        self.address_mode_indirect_y_indexed(bus);
        self.compare(bus);
        self.sleep_cycles += 5;
        self.sleep_cycles += self.crossed_page_boundary as u8;
    }

    fn run_E1(&mut self, bus: &B) {
        self.address_mode_x_indexed_indirect(bus);
        self.subtract_with_carry(bus);
        self.sleep_cycles += 6;
    }

    fn run_F1(&mut self, bus: &B) {
        self.address_mode_indirect_y_indexed(bus);
        self.subtract_with_carry(bus);
        self.sleep_cycles += 5;
        self.sleep_cycles += self.crossed_page_boundary as u8;
    }

    // 02 - illegal
    // 12 - illegal
    // 22 - illegal
    // 32 - illegal
    // 42 - illegal
    // 52 - illegal
    // 62 - illegal
    // 72 - illegal
    // 82 - illegal
    // 92 - illegal

    fn run_A2(&mut self, bus: &B) {
        self.address_mode_immediate(bus);
        self.load_x(bus);
        self.sleep_cycles += 2;
    }

    // B2 - illegal
    // C2 - illegal
    // D2 - illegal
    // E2 - illegal
    // F2 - illegal
    // 03 - illegal
    // 13 - illegal
    // 23 - illegal
    // 33 - illegal
    // 43 - illegal
    // 53 - illegal
    // 63 - illegal
    // 73 - illegal
    // 83 - illegal
    // 93 - illegal
    // A3 - illegal
    // B3 - illegal
    // C3 - illegal
    // D3 - illegal
    // E3 - illegal
    // F3 - illegal
    // 04 - illegal
    // 14 - illegal

    fn run_24(&mut self, bus: &B) {
        self.address_mode_zeropage(bus);
        self.bit_test(bus);
        self.sleep_cycles += 3;
    }

    // 34 - illegal
    // 44 - illegal
    // 54 - illegal
    // 64 - illegal
    // 74 - illegal

    fn run_84(&mut self, bus: &B) {
        self.address_mode_zeropage(bus);
        self.store_y(bus);
        self.sleep_cycles += 3;
    }

    fn run_94(&mut self, bus: &B) {
        self.address_mode_zeropage_x_indexed(bus);
        self.store_y(bus);
        self.sleep_cycles += 4;
    }

    fn run_A4(&mut self, bus: &B) {
        self.address_mode_zeropage(bus);
        self.load_y(bus);
        self.sleep_cycles += 3;
    }

    fn run_B4(&mut self, bus: &B) {
        self.address_mode_zeropage_x_indexed(bus);
        self.load_y(bus);
        self.sleep_cycles += 4;
    }

    fn run_C4(&mut self, bus: &B) {
        self.address_mode_zeropage(bus);
        self.compare_with_y(bus);
        self.sleep_cycles += 3;
    }

    // D4 - illegal

    fn run_E4(&mut self, bus: &B) {
        self.address_mode_zeropage(bus);
        self.compare_with_y(bus);
        self.sleep_cycles += 3;
    }

    // F4 - illegal

    fn run_05(&mut self, bus: &B) {
        self.address_mode_zeropage(bus);
        self.or_with_accumulator(bus);
        self.sleep_cycles += 3;
    }

    fn run_15(&mut self, bus: &B) {
        self.address_mode_zeropage_x_indexed(bus);
        self.or_with_accumulator(bus);
        self.sleep_cycles += 4;
    }

    fn run_25(&mut self, bus: &B) {
        self.address_mode_zeropage(bus);
        self.and(bus);
        self.sleep_cycles += 3;
    }

    fn run_35(&mut self, bus: &B) {
        self.address_mode_zeropage_x_indexed(bus);
        self.and(bus);
        self.sleep_cycles += 4;
    }

    fn run_45(&mut self, bus: &B) {
        self.address_mode_zeropage(bus);
        self.exclusive_or(bus);
        self.sleep_cycles += 3;
    }

    fn run_55(&mut self, bus: &B) {
        self.address_mode_zeropage_x_indexed(bus);
        self.exclusive_or(bus);
        self.sleep_cycles += 4;
    }

    fn run_65(&mut self, bus: &B) {
        self.address_mode_zeropage(bus);
        self.add_with_carry(bus);
        self.sleep_cycles += 3;
    }

    fn run_75(&mut self, bus: &B) {
        self.address_mode_zeropage_x_indexed(bus);
        self.add_with_carry(bus);
        self.sleep_cycles += 4;
    }

    fn run_85(&mut self, bus: &B) {
        self.address_mode_zeropage(bus);
        self.store_accumulator(bus);
        self.sleep_cycles += 3;
    }

    fn run_95(&mut self, bus: &B) {
        self.address_mode_zeropage_x_indexed(bus);
        self.store_accumulator(bus);
        self.sleep_cycles += 4;
    }

    fn run_A5(&mut self, bus: &B) {
        self.address_mode_zeropage(bus);
        self.load_accumulator(bus);
        self.sleep_cycles += 3;
    }

    fn run_B5(&mut self, bus: &B) {
        self.address_mode_zeropage_x_indexed(bus);
        self.load_accumulator(bus);
        self.sleep_cycles += 4;
    }

    fn run_C5(&mut self, bus: &B) {
        self.address_mode_zeropage(bus);
        self.compare(bus);
        self.sleep_cycles += 3;
    }

    fn run_D5(&mut self, bus: &B) {
        self.address_mode_zeropage_x_indexed(bus);
        self.compare(bus);
        self.sleep_cycles += 4;
    }

    fn run_E5(&mut self, bus: &B) {
        self.address_mode_zeropage(bus);
        self.subtract_with_carry(bus);
        self.sleep_cycles += 3;
    }

    fn run_F5(&mut self, bus: &B) {
        self.address_mode_zeropage_x_indexed(bus);
        self.subtract_with_carry(bus);
        self.sleep_cycles += 4;
    }

    fn run_06(&mut self, bus: &B) {
        self.address_mode_zeropage(bus);
        self.arithmetic_shift_left(bus);
        self.sleep_cycles += 5;
    }

    fn run_16(&mut self, bus: &B) {
        self.address_mode_zeropage_x_indexed(bus);
        self.arithmetic_shift_left(bus);
        self.sleep_cycles += 6;
    }

    fn run_26(&mut self, bus: &B) {
        self.address_mode_zeropage(bus);
        self.rotate_left(bus);
        self.sleep_cycles += 5;
    }

    fn run_36(&mut self, bus: &B) {
        self.address_mode_zeropage_x_indexed(bus);
        self.rotate_left(bus);
        self.sleep_cycles += 6;
    }

    fn run_46(&mut self, bus: &B) {
        self.address_mode_zeropage(bus);
        self.logical_shift_right(bus);
        self.sleep_cycles += 5;
    }

    fn run_56(&mut self, bus: &B) {
        self.address_mode_zeropage_x_indexed(bus);
        self.logical_shift_right(bus);
        self.sleep_cycles += 6;
    }

    fn run_66(&mut self, bus: &B) {
        self.address_mode_zeropage(bus);
        self.rotate_right(bus);
        self.sleep_cycles += 5;
    }

    fn run_76(&mut self, bus: &B) {
        self.address_mode_zeropage_x_indexed(bus);
        self.rotate_right(bus);
        self.sleep_cycles += 6;
    }

    fn run_86(&mut self, bus: &B) {
        self.address_mode_zeropage(bus);
        self.store_x(bus);
        self.sleep_cycles += 3;
    }

    fn run_96(&mut self, bus: &B) {
        self.address_mode_zeropage_y_indexed(bus);
        self.store_x(bus);
        self.sleep_cycles += 4;
    }

    fn run_A6(&mut self, bus: &B) {
        self.address_mode_zeropage(bus);
        self.load_x(bus);
        self.sleep_cycles += 3;
    }

    fn run_B6(&mut self, bus: &B) {
        self.address_mode_zeropage_y_indexed(bus);
        self.load_x(bus);
        self.sleep_cycles += 4;
    }

    fn run_C6(&mut self, bus: &B) {
        self.address_mode_zeropage(bus);
        self.decrement(bus);
        self.sleep_cycles += 5;
    }

    fn run_D6(&mut self, bus: &B) {
        self.address_mode_zeropage_x_indexed(bus);
        self.decrement(bus);
        self.sleep_cycles += 6;
    }

    fn run_E6(&mut self, bus: &B) {
        self.address_mode_zeropage(bus);
        self.increment(bus);
        self.sleep_cycles += 5;
    }

    fn run_F6(&mut self, bus: &B) {
        self.address_mode_zeropage_x_indexed(bus);
        self.increment(bus);
        self.sleep_cycles += 6;
    }

    // 07 - illegal
    // 17 - illegal
    // 27 - illegal
    // 37 - illegal
    // 47 - illegal
    // 57 - illegal
    // 67 - illegal
    // 77 - illegal
    // 87 - illegal
    // 97 - illegal
    // A7 - illegal
    // B7 - illegal
    // C7 - illegal
    // D7 - illegal
    // E7 - illegal
    // F7 - illegal

    fn run_08(&mut self, bus: &B) {
        self.address_mode_implied(bus);
        self.push_processor_status(bus);
        self.sleep_cycles += 3;
    }

    fn run_18(&mut self, bus: &B) {
        self.address_mode_implied(bus);
        self.clear_carry(bus);
        self.sleep_cycles += 2;
    }

    fn run_28(&mut self, bus: &B) {
        self.address_mode_implied(bus);
        self.pull_processor_status(bus);
        self.sleep_cycles += 4;
    }

    fn run_38(&mut self, bus: &B) {
        self.address_mode_implied(bus);
        self.set_carry(bus);
        self.sleep_cycles += 2;
    }

    fn run_48(&mut self, bus: &B) {
        self.address_mode_implied(bus);
        self.push_accumulator(bus);
        self.sleep_cycles += 3;
    }

    fn run_58(&mut self, bus: &B) {
        self.address_mode_implied(bus);
        self.clear_interrupt_disable(bus);
        self.sleep_cycles += 2;
    }

    fn run_68(&mut self, bus: &B) {
        self.address_mode_implied(bus);
        self.pull_accumulator(bus);
        self.sleep_cycles += 4;
    }

    fn run_78(&mut self, bus: &B) {
        self.address_mode_implied(bus);
        self.set_interrupt_disable(bus);
        self.sleep_cycles += 2;
    }

    fn run_88(&mut self, bus: &B) {
        self.address_mode_implied(bus);
        self.decrement_y(bus);
        self.sleep_cycles += 2;
    }

    fn run_98(&mut self, bus: &B) {
        self.address_mode_implied(bus);
        self.transfer_y_to_accumulator(bus);
        self.sleep_cycles += 2;
    }

    fn run_A8(&mut self, bus: &B) {
        self.address_mode_implied(bus);
        self.transfer_accumulator_to_y(bus);
        self.sleep_cycles += 2;
    }

    fn run_B8(&mut self, bus: &B) {
        self.address_mode_implied(bus);
        self.clear_overflow(bus);
        self.sleep_cycles += 2;
    }

    fn run_C8(&mut self, bus: &B) {
        self.address_mode_implied(bus);
        self.increment_y(bus);
        self.sleep_cycles += 2;
    }

    fn run_D8(&mut self, bus: &B) {
        self.address_mode_implied(bus);
        self.clear_decimal(bus);
        self.sleep_cycles += 2;
    }

    fn run_E8(&mut self, bus: &B) {
        self.address_mode_implied(bus);
        self.increment_x(bus);
        self.sleep_cycles += 2;
    }

    fn run_F8(&mut self, bus: &B) {
        self.address_mode_implied(bus);
        self.set_decimal(bus);
        self.sleep_cycles += 2;
    }

    fn run_09(&mut self, bus: &B) {
        self.address_mode_immediate(bus);
        self.or_with_accumulator(bus);
        self.sleep_cycles += 2;
    }

    fn run_19(&mut self, bus: &B) {
        self.address_mode_absolute_y_indexed(bus);
        self.or_with_accumulator(bus);
        self.sleep_cycles += 4;
        self.sleep_cycles += self.crossed_page_boundary as u8;
    }

    fn run_29(&mut self, bus: &B) {
        self.address_mode_immediate(bus);
        self.and(bus);
        self.sleep_cycles += 2;
    }

    fn run_39(&mut self, bus: &B) {
        self.address_mode_absolute_y_indexed(bus);
        self.and(bus);
        self.sleep_cycles += 4;
        self.sleep_cycles += self.crossed_page_boundary as u8;
    }

    fn run_49(&mut self, bus: &B) {
        self.address_mode_immediate(bus);
        self.exclusive_or(bus);
        self.sleep_cycles += 2;
    }

    fn run_59(&mut self, bus: &B) {
        self.address_mode_absolute_y_indexed(bus);
        self.exclusive_or(bus);
        self.sleep_cycles += 4;
        self.sleep_cycles += self.crossed_page_boundary as u8;
    }

    fn run_69(&mut self, bus: &B) {
        self.address_mode_immediate(bus);
        self.add_with_carry(bus);
        self.sleep_cycles += 2;
    }

    fn run_79(&mut self, bus: &B) {
        self.address_mode_absolute_y_indexed(bus);
        self.add_with_carry(bus);
        self.sleep_cycles += 4;
        self.sleep_cycles += self.crossed_page_boundary as u8;
    }

    // 89 - illegal

    fn run_99(&mut self, bus: &B) {
        self.address_mode_absolute_y_indexed(bus);
        self.store_accumulator(bus);
        self.sleep_cycles += 5;
    }

    fn run_A9(&mut self, bus: &B) {
        self.address_mode_immediate(bus);
        self.load_accumulator(bus);
        self.sleep_cycles += 2;
    }

    fn run_B9(&mut self, bus: &B) {
        self.address_mode_absolute_y_indexed(bus);
        self.load_accumulator(bus);
        self.sleep_cycles += 4;
        self.sleep_cycles += self.crossed_page_boundary as u8;
    }

    fn run_C9(&mut self, bus: &B) {
        self.address_mode_immediate(bus);
        self.compare(bus);
        self.sleep_cycles += 2;
    }

    fn run_D9(&mut self, bus: &B) {
        self.address_mode_absolute_y_indexed(bus);
        self.compare(bus);
        self.sleep_cycles += 4;
        self.sleep_cycles += self.crossed_page_boundary as u8;
    }

    fn run_E9(&mut self, bus: &B) {
        self.address_mode_immediate(bus);
        self.subtract_with_carry(bus);
        self.sleep_cycles += 2;
    }

    fn run_F9(&mut self, bus: &B) {
        self.address_mode_absolute_y_indexed(bus);
        self.subtract_with_carry(bus);
        self.sleep_cycles += 4;
        self.sleep_cycles += self.crossed_page_boundary as u8;
    }

    fn run_0A(&mut self, bus: &B) {
        self.address_mode_implied(bus);
        self.arithmetic_shift_left(bus);
        self.sleep_cycles += 2;
    }

    // 1A - illegal

    fn run_2A(&mut self, bus: &B) {
        self.address_mode_implied(bus);
        self.rotate_left(bus);
        self.sleep_cycles += 2;
    }

    // 3A - illegal

    fn run_4A(&mut self, bus: &B) {
        self.address_mode_implied(bus);
        self.logical_shift_right(bus);
        self.sleep_cycles += 2;
    }

    // 5A - illegal

    fn run_6A(&mut self, bus: &B) {
        self.address_mode_implied(bus);
        self.rotate_right(bus);
        self.sleep_cycles += 2;
    }

    // 7A - illegal

    fn run_8A(&mut self, bus: &B) {
        self.address_mode_implied(bus);
        self.transfer_x_to_accumulator(bus);
        self.sleep_cycles += 2;
    }

    fn run_9A(&mut self, bus: &B) {
        self.address_mode_implied(bus);
        self.transfer_x_to_stack_pointer(bus);
        self.sleep_cycles += 2;
    }

    fn run_AA(&mut self, bus: &B) {
        self.address_mode_implied(bus);
        self.transfer_accumulator_to_x(bus);
        self.sleep_cycles += 2;
    }

    fn run_BA(&mut self, bus: &B) {
        self.address_mode_implied(bus);
        self.transfer_stack_pointer_to_x(bus);
        self.sleep_cycles += 2;
    }

    fn run_CA(&mut self, bus: &B) {
        self.address_mode_implied(bus);
        self.decrement_x(bus);
        self.sleep_cycles += 2;
    }

    // DA - illegal

    fn run_EA(&mut self, bus: &B) {
        self.address_mode_implied(bus);
        self.no_operation(bus);
        self.sleep_cycles += 2;
    }

    // FA - illegal
    // 0B - illegal
    // 1B - illegal
    // 2B - illegal
    // 3B - illegal
    // 4B - illegal
    // 5B - illegal
    // 6B - illegal
    // 7B - illegal
    // 8B - illegal
    // 9B - illegal
    // AB - illegal
    // BB - illegal
    // CB - illegal
    // DB - illegal
    // EB - illegal
    // FB - illegal
    // 0C - illegal
    // 1C - illegal

    fn run_2C(&mut self, bus: &B) {
        self.address_mode_absolute(bus);
        self.bit_test(bus);
        self.sleep_cycles += 4;
    }

    // 3C - illegal

    fn run_4C(&mut self, bus: &B) {
        self.address_mode_absolute(bus);
        self.jump(bus);
        self.sleep_cycles += 3;
    }

    // 5C - illegal

    fn run_6C(&mut self, bus: &B) {
        self.address_mode_indirect(bus);
        self.jump(bus);
        self.sleep_cycles += 5;
    }

    // 7C - illegal

    fn run_8C(&mut self, bus: &B) {
        self.address_mode_absolute(bus);
        self.store_y(bus);
        self.sleep_cycles += 4;
    }

    // 9C - illegal

    fn run_AC(&mut self, bus: &B) {
        self.address_mode_absolute(bus);
        self.load_y(bus);
        self.sleep_cycles += 4;
    }

    fn run_BC(&mut self, bus: &B) {
        self.address_mode_absolute_x_indexed(bus);
        self.load_y(bus);
        self.sleep_cycles += 4;
        self.sleep_cycles += self.crossed_page_boundary as u8;
    }

    fn run_CC(&mut self, bus: &B) {
        self.address_mode_absolute(bus);
        self.compare_with_y(bus);
        self.sleep_cycles += 4;
    }

    // DC - illegal

    fn run_EC(&mut self, bus: &B) {
        self.address_mode_absolute(bus);
        self.compare_with_x(bus);
        self.sleep_cycles += 4;
    }

    // FC - illegal

    fn run_0D(&mut self, bus: &B) {
        self.address_mode_absolute(bus);
        self.or_with_accumulator(bus);
        self.sleep_cycles += 4;
    }

    fn run_1D(&mut self, bus: &B) {
        self.address_mode_absolute_x_indexed(bus);
        self.or_with_accumulator(bus);
        self.sleep_cycles += 4;
        self.sleep_cycles += self.crossed_page_boundary as u8;
    }

    fn run_2D(&mut self, bus: &B) {
        self.address_mode_absolute(bus);
        self.and(bus);
        self.sleep_cycles += 4;
    }

    fn run_3D(&mut self, bus: &B) {
        self.address_mode_absolute_x_indexed(bus);
        self.and(bus);
        self.sleep_cycles += 4;
        self.sleep_cycles += self.crossed_page_boundary as u8;
    }

    fn run_4D(&mut self, bus: &B) {
        self.address_mode_absolute(bus);
        self.exclusive_or(bus);
        self.sleep_cycles += 4;
    }

    fn run_5D(&mut self, bus: &B) {
        self.address_mode_absolute_x_indexed(bus);
        self.exclusive_or(bus);
        self.sleep_cycles += 4;
        self.sleep_cycles += self.crossed_page_boundary as u8;
    }

    fn run_6D(&mut self, bus: &B) {
        self.address_mode_absolute(bus);
        self.add_with_carry(bus);
        self.sleep_cycles += 4;
    }

    fn run_7D(&mut self, bus: &B) {
        self.address_mode_absolute_x_indexed(bus);
        self.add_with_carry(bus);
        self.sleep_cycles += 4;
        self.sleep_cycles += self.crossed_page_boundary as u8;
    }

    fn run_8D(&mut self, bus: &B) {
        self.address_mode_absolute(bus);
        self.store_accumulator(bus);
        self.sleep_cycles += 4;
    }

    fn run_9D(&mut self, bus: &B) {
        self.address_mode_absolute_x_indexed(bus);
        self.store_accumulator(bus);
        self.sleep_cycles += 5;
    }

    fn run_AD(&mut self, bus: &B) {
        self.address_mode_absolute(bus);
        self.load_accumulator(bus);
        self.sleep_cycles += 4;
    }

    fn run_BD(&mut self, bus: &B) {
        self.address_mode_absolute_x_indexed(bus);
        self.load_accumulator(bus);
        self.sleep_cycles += 4;
        self.sleep_cycles += self.crossed_page_boundary as u8;
    }

    fn run_CD(&mut self, bus: &B) {
        self.address_mode_absolute(bus);
        self.compare(bus);
        self.sleep_cycles += 4;
    }

    fn run_DD(&mut self, bus: &B) {
        self.address_mode_absolute_x_indexed(bus);
        self.compare(bus);
        self.sleep_cycles += 4;
        self.sleep_cycles += self.crossed_page_boundary as u8;
    }

    fn run_ED(&mut self, bus: &B) {
        self.address_mode_absolute(bus);
        self.subtract_with_carry(bus);
        self.sleep_cycles += 4;
    }

    fn run_FD(&mut self, bus: &B) {
        self.address_mode_absolute_x_indexed(bus);
        self.subtract_with_carry(bus);
        self.sleep_cycles += 4;
        self.sleep_cycles += self.crossed_page_boundary as u8;
    }

    fn run_0E(&mut self, bus: &B) {
        self.address_mode_absolute(bus);
        self.arithmetic_shift_left(bus);
        self.sleep_cycles += 6;
    }

    fn run_1E(&mut self, bus: &B) {
        self.address_mode_absolute_x_indexed(bus);
        self.arithmetic_shift_left(bus);
        self.sleep_cycles += 7;
    }

    fn run_2E(&mut self, bus: &B) {
        self.address_mode_absolute(bus);
        self.rotate_left(bus);
        self.sleep_cycles += 6;
    }

    fn run_3E(&mut self, bus: &B) {
        self.address_mode_absolute_x_indexed(bus);
        self.rotate_left(bus);
        self.sleep_cycles += 7;
    }

    fn run_4E(&mut self, bus: &B) {
        self.address_mode_absolute(bus);
        self.logical_shift_right(bus);
        self.sleep_cycles += 6;
    }

    fn run_5E(&mut self, bus: &B) {
        self.address_mode_absolute_x_indexed(bus);
        self.logical_shift_right(bus);
        self.sleep_cycles += 7;
    }

    fn run_6E(&mut self, bus: &B) {
        self.address_mode_absolute(bus);
        self.rotate_right(bus);
        self.sleep_cycles += 6;
    }

    fn run_7E(&mut self, bus: &B) {
        self.address_mode_absolute_x_indexed(bus);
        self.rotate_right(bus);
        self.sleep_cycles += 7;
    }

    fn run_8E(&mut self, bus: &B) {
        self.address_mode_absolute(bus);
        self.store_x(bus);
        self.sleep_cycles += 4;
    }

    // 9E - illegal

    fn run_AE(&mut self, bus: &B) {
        self.address_mode_absolute(bus);
        self.load_x(bus);
        self.sleep_cycles += 4;
    }

    fn run_BE(&mut self, bus: &B) {
        self.address_mode_absolute_y_indexed(bus);
        self.load_x(bus);
        self.sleep_cycles += 4;
        self.sleep_cycles += self.crossed_page_boundary as u8;
    }

    fn run_CE(&mut self, bus: &B) {
        self.address_mode_absolute(bus);
        self.decrement(bus);
        self.sleep_cycles += 3;
    }

    fn run_DE(&mut self, bus: &B) {
        self.address_mode_absolute_x_indexed(bus);
        self.decrement(bus);
        self.sleep_cycles += 7;
    }

    fn run_EE(&mut self, bus: &B) {
        self.address_mode_absolute(bus);
        self.increment(bus);
        self.sleep_cycles += 6;
    }

    fn run_FE(&mut self, bus: &B) {
        self.address_mode_absolute_x_indexed(bus);
        self.increment(bus);
        self.sleep_cycles += 7;
    }

    // 0F - illegal
    // 1F - illegal
    // 2F - illegal
    // 3F - illegal
    // 4F - illegal
    // 5F - illegal
    // 6F - illegal
    // 7F - illegal
    // 8F - illegal
    // 9F - illegal
    // AF - illegal
    // BF - illegal
    // CF - illegal
    // DF - illegal
    // EF - illegal
    // FF - illegal

    fn run_xx(&mut self, bus: &B) {
        eprintln!("Illegal opcode found ({:#02}). Running noop.", self.opcode);
        self.program_counter = self.program_counter.wrapping_add(1);
        self.sleep_cycles = 2;
    }
}

impl<B> std::fmt::Debug for Cpu<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A={:02X} X={:02X} Y={:02X} PC={:04X} SP={:02X} SR=(N={} V={} _={} B={} D={} I={} Z={} C={})",
            self.accumulator,
            self.x_index,
            self.y_index,
            self.program_counter,
            self.stack_pointer,
            self.status_register.get_negative() as u8,
            self.status_register.get_overflow() as u8,
            self.status_register.get_ignored() as u8,
            self.status_register.get_break() as u8,
            self.status_register.get_decimal() as u8,
            self.status_register.get_interrupt() as u8,
            self.status_register.get_zero() as u8,
            self.status_register.get_carry() as u8,
        )
    }
}

fn u8_to_i8(integer: u8) -> i8 {
    integer as i16 as i8
}

fn u16_address_offset(address: u16, by: i8) -> u16 {
    if by >= 0 {
        address.wrapping_add(by as u16)
    } else {
        address.wrapping_sub(-(by as i16) as u16)
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct StatusRegister(u8);

macro_rules! flag_methods {
    ($get_flag_name:ident,$set_flag_name:ident,$bit:expr) => {
        fn $get_flag_name(&self) -> bool {
            (self.0 & (1 << $bit)) > 0
        }
        fn $set_flag_name(&mut self, new: bool) {
            self.0 = (self.0 & !(1 << $bit)) | ((new as u8) << $bit);
        }
    };
}

impl StatusRegister {
    fn new() -> Self {
        Self(0)
    }
    flag_methods!(get_negative,  set_negative,  7);
    flag_methods!(get_overflow,  set_overflow,  6);
    flag_methods!(get_ignored,   set_ignored,   5);
    flag_methods!(get_break,     set_break,     4);
    flag_methods!(get_decimal,   set_decimal,   3);
    flag_methods!(get_interrupt, set_interrupt, 2);
    flag_methods!(get_zero,      set_zero,      1);
    flag_methods!(get_carry,     set_carry,     0);
}
