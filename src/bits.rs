use crate::operation::*;
use bitflags::bitflags;
bitflags! {
    #[derive(PartialEq, Eq)]
    pub struct IMMEDIATE_TO_REGISTER_OR_MEMORY_RESULTS: u8 {
        // TODO: figure out if the second last bit matters if last (w) is set to 0. (Same applies to all instructions here)
        const MOV_MOVE_8 = 0b_11_000_110;
        const MOV_MOVE_16 = 0b_11_000_111;
        const ADD_MOVE_8 = 0b_10_000_010;
        const ADD_MOVE_16 = 0b_10_000_011;
        const SUB_OR_CMP_MOVE_8 = 0b_10_000_010;
        const SUB_OR_CMP_MOVE_16 = 0b_10_000_011;
    }

    #[derive(PartialEq, Eq)]
    // This is fetched from the second bytes reg field which is 3 bits.
    pub struct IMMEDIATE_TO_REGISTER_OR_MEMORY_REG_RESULTS: u8 {
        // this is either mov or add because their reg fields are the same.
        // the difference is determined by the first byte and we're going to handle the first byte first in this case.
        const MOV_OR_ADD_RESULT = 0b_00_000_000;
        //
        const SUB_RESULT = 0b_00_101_000;
        const CMP_RESULT = 0b_00_111_000;
    }

    #[derive(PartialEq, Eq)]
    pub struct MEMORY_TO_REGISTER_VICA_VERCA_MNEMONIC_MASK_RESULTS: u8{
       // TODO: there is some other alternative we have to add here. Figure out what it is.
       // used on the first byte.
       const MOV = 0b_00001000;
       const ADD = 0b_00000000;
       const SUB = 0b_00101000;
       const CMP = 0b_00111000;
    }

    #[derive(PartialEq, Eq)]
    pub struct IMMEDIATE_TO_REGISTER_REG_FIELD_MASK_RESULTS: u8 {
        const BX_PLUS_SI = 0b00_000_000;
        const BX_PLUS_DI = 0b00_000_001;
        const BP_PLUS_SI = 0b00_000_010;
        const BP_PLUS_DI = 0b00_000_011;
        const SI = 0b_00_000_100;
        const DI = 0b00_000_101;
        const BP_OR_NONE = 0b00_000_110;
        const BX = 0b_00_000_111;
    }
    #[derive(PartialEq, Eq)]
    // these results are used to determine the mode that is going to be done for example:
    // Register to register, memory to register, immediate value to register etc.
    pub struct MOD_MODE_RESULTS: u8 {
        const REGISTER_MODE = 0b_11_000_000;
        const MEMORY_MODE_8 = 0b_01_000_000;
        const MEMORY_MODE_16 = 0b_10_000_000;
        const MEMORY_MODE = 0b_00_000_000; // (No displacement when r/m is not 110, else its a 16 bit placement).
    }

    #[derive(PartialEq, Eq)]
    pub struct IMMEDIATE_TO_REGISTER_MASK_RESULTS: u8 {
        const IMMEDIATE_TO_REGISTER_16= 0b_1011_1000;
        const IMMEDIATE_TO_REGISTER_8 = 0b_1011_0000;
    }

    #[derive(PartialEq, Eq)]
    pub struct REGISTER_TO_OR_MEMORY_REG_MASK_RESULTS: u8 {
        const AX_OR_AL = 0b_00_000_000;
        const CX_OR_CL = 0b_00_001_000;
        const DX_OR_DL = 0b_00_010_000;
        const BX_OR_BL = 0b_00_011_000;
        const SP_OR_AH = 0b_00_100_000;
        const BP_OR_CH = 0b_00_101_000;
        const SI_OR_DH = 0b_00_110_000;
        const DI_OR_BH = 0b_00_111_000;
    }

    #[derive(PartialEq, Eq)]
    pub struct IMMEDIATE_TO_REGISTER_MODE_REG_MASK_RESULTS: u8 {
        const AX_OR_AL = 0b_00_000_000;
        const CX_OR_CL = 0b_00_000_001;
        const DX_OR_DL = 0b_00_000_010;
        const BX_OR_BL = 0b_00_000_011;
        const SP_OR_AH = 0b_00_000_100;
        const BP_OR_CH = 0b_00_000_101;
        const SI_OR_DH = 0b_00_000_110;
        const DI_OR_BH = 0b_00_000_111;
    }

    pub struct OPERATIONS: u8 {
        const IMMEDIATE_TO_REGISTER_MASK = 0b_11111000;
        const IMMEDIATE_TO_REGISTER_OR_MEMORY = 0b_11111111; // This actually relies on the second byte also since its mod field determines which mnemonic is being used.
    }

    pub struct FIRST_BYTE: u8 {
        const IMMEDIATE_OR_REGISTER_MODE_REG_MASK = 0b_00_000_111;
        const D_BIT_MASK = 0b000000_10; // This is the D bit specified after the instruction operand. It's responsible for specifying the destination and source register.
        const IMMEDIATE_TO_REGISTER_W_MASK = 0b_0000_1000; // This is the W bit of a memory to register, register to memory and register to register move and it's responsible for determining the size of the registers (8 or 16 bit).
        const MEMORY_TO_REGISTER_VICA_VERCA_W_MASK = 0b000000_01; // This is the W bit of a memory to register, register to memory and register to register move and it's responsible for determining the size of the registers (8 or 16 bit).
        const IMMEDIATE_TO_REGISTER_REG_FIELD_MASK = 0b_00_000_111; // this is used to get the contents of the REG field if it's present in the first byte.
        const MEMORY_TO_REGISTER_VICA_VERCA_MNEMONIC_MASK = 0b_00111000; // To determine what mnemonic is being used.
        const IMMEDIATE_TO_REGISTER_OR_MEMORY_W_BIT = 0b_00000001; // This instruction actually relies on the second byte also since its mod field determines which mnemonic is being used.
        const IMMEDIATE_TO_REGISTER_OR_MEMORY_S_BIT = 0b_00000010; // If this (S bit) is set to 1, and W = 1 then
    }

    pub struct SECOND_BYTE: u8 {
        const REGISTER_TO_OR_MEMORY_REG_MASK = 0b_00_111_000; // this is only used for immediate with register / memory and register to register / memory to register and vica verca operations.
        const MOD_MASK = 0b_11_000_000;
        const RM_MASK = 0b_00_000_111; // this is used to get the contents of the R/M field
    }

    pub struct MOD_RESULTS: u8 {
        const REGISTER_MODE = 0b_11_000_000;
        const MEMORY_MODE = 0b_00_000_000; // No displacement follows expect 16-bit when r/m = 110
        const MEMORY_MODE_8_BIT_DISPLACEMENT = 0b_01_000_000;
        const MEMORY_MODE_16_BIT_DISPLACEMENT = 0b_10_000_000;
    }

    #[derive(PartialEq)]
    pub struct REG_OR_MEMORY_WITH_REGISTER_MNEMONIC_RESULTS: u8 {
        const MOV = 0b_10001000;
        const ADD = 0b_00000000;
        const SUB = 0b_00101000;
        const CMP = 0b_00111000;
    }

    #[derive(PartialEq)]
    pub struct IMMEDIATE_TO_MEMORY_OR_REGISTER: u8 {
        const MOV = 0b_1100_0100;

        // SUB and CMP are the same in the first byte, the second fields REG field determines the difference.
        const SUB_OR_CMP_OR_ADD = 0b_1000_0000;
    }


    pub struct MASKS: u8 {
        const MOD = 0b_11_000_000;

        // r/m is basically always in the same bits.
        const RM = 0b_00_000_111;

        // REG is usually in the same place. There are some edge cases however for example the mov immediate_to_register.
        const REG = 0b_00_111_000;

        const MOV_IMMEDIATE_TO_REGISTER_W_BIT = 0b_00_001_000; // there is one exception i saw with the w bit and thats mov immediate to register.

        const IMMEDIATE_TO_MEMORY_OR_REGISTER_S_BIT = 0b_00_000_010; // TODO: what is this exactly for?

        const W_BIT = 0b_00_000_001; // this is used for basically everything.

        //
        const MOV_IMMEDIATE_TO_REGISTER = 0b_10110000; // determines the mov immediate to register operation.

        const REG_OR_MEMORY_WITH_REGISTER = 0b_10111000; // determines if it's a reg/memory and register to either operation.

        const IMMEDIATE_TO_MEMORY_OR_REGISTER = 0b_11111100; // determines if the operation is an immediate to register or immediate to memory operation.
    }

}

impl REG_OR_MEMORY_WITH_REGISTER_MNEMONIC_RESULTS {
    pub fn convert_to_mnemonic(res: u8) -> &'static str {
        if res == REG_OR_MEMORY_WITH_REGISTER_MNEMONIC_RESULTS::MOV.bits() {
            return "mov";
        } else if res == REG_OR_MEMORY_WITH_REGISTER_MNEMONIC_RESULTS::ADD.bits() {
            return "add";
        } else if res == REG_OR_MEMORY_WITH_REGISTER_MNEMONIC_RESULTS::SUB.bits() {
            return "sub";
        } else if res == REG_OR_MEMORY_WITH_REGISTER_MNEMONIC_RESULTS::CMP.bits() {
            return "cmp";
        } else {
            panic!("unknown bit pattern: {:08b}.", res)
        }
    }
}

impl IMMEDIATE_TO_MEMORY_OR_REGISTER {
    // the IMMEDIATE_TO_MEMORY_OR_REGISTER convert to mnemonic function relies on the second bytes reg
    // to determine the difference between sub and cmp
    pub fn convert_to_mnemonic(res: u8, second_byte: u8) -> &'static str {
        if res == IMMEDIATE_TO_MEMORY_OR_REGISTER::MOV.bits() {
            return "mov";
        } else if res == IMMEDIATE_TO_MEMORY_OR_REGISTER::SUB_OR_CMP_OR_ADD.bits() {
            let REG = second_byte & MASKS::REG.bits();
            if REG == 0b_00_000_000 {
                return "add";
            } else if REG == 0b_00_101_000 {
                return "sub";
            } else if REG == 0b_00_111_000 {
                return "cmp";
            } else {
                panic!("Unsupported reg result: {:08b}", REG)
            }
        } else {
            panic!("unknown bit pattern: {:08b}.", res)
        }
    }
}

impl MOD_RESULTS {
    pub fn convert_to_operation(r: u8) -> Operation {
        if r == MOD_RESULTS::MEMORY_MODE.bits() {
            return Operation::MEMORY_MODE_NONE;
        } else if r == MOD_RESULTS::MEMORY_MODE_8_BIT_DISPLACEMENT.bits() {
            return Operation::MEMORY_MODE_8_BIT_DISPLACEMENT;
        } else if r == MOD_RESULTS::MEMORY_MODE_16_BIT_DISPLACEMENT.bits() {
            return Operation::MEMORY_MODE_16_BIT_DISPLACEMENT;
        } else if r == MOD_RESULTS::REGISTER_MODE.bits() {
            return Operation::REGISTER_MODE;
        } else {
            panic!("Unsupported operation: {:08b}", r)
        }
    }
}
