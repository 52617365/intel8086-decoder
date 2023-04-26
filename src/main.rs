mod bits;
mod operation;
use bits::*;
use operation::*;

use core::panic;
use std::{env, fs};

// Ideas:
// Should for example the registers be coupled with the Operation enum? E.g. the reg and r/m registers would be wrapped into the enum itself, this would
// allow stuff to not be so independent from eachother.

struct Instruction {
    op: Operation,
    mnemonic: &'static str,
}

// W bit determines the size between 8 and 16-bits, the w bit is at different places depending on the instruction.
fn is_word_size(first_byte: u8, op: Operation) -> bool {
    if op == Operation::IMMEDIATE_TO_REGISTER_MOV_16 || op == Operation::IMMEDIATE_TO_REGISTER_MOV_8
    {
        return first_byte & MASKS::MOV_IMMEDIATE_TO_REGISTER_W_BIT.bits() != 0;
    } else {
        return first_byte & MASKS::W_BIT.bits() != 0;
    }
}

fn is_mov_immediate_to_register(first_byte: u8) -> bool {
    return first_byte & MASKS::MOV_IMMEDIATE_TO_REGISTER.bits()
        == MASKS::MOV_IMMEDIATE_TO_REGISTER.bits();
}

// checks to see if the operation used is is_register_or_memory_with_register
fn is_register_or_memory_with_register(first_byte: u8) -> bool {
    let mask_res = first_byte & MASKS::REG_OR_MEMORY_WITH_REGISTER.bits();

    if mask_res == REG_OR_MEMORY_WITH_REGISTER_MNEMONIC_RESULTS::MOV.bits()
        || mask_res == REG_OR_MEMORY_WITH_REGISTER_MNEMONIC_RESULTS::ADD.bits()
        || mask_res == REG_OR_MEMORY_WITH_REGISTER_MNEMONIC_RESULTS::SUB.bits()
        || mask_res == REG_OR_MEMORY_WITH_REGISTER_MNEMONIC_RESULTS::CMP.bits()
    {
        return true;
    } else {
        return false;
    }
}

// checks to see if the operation used is immediate_to_memory_or_register
fn is_immediate_to_memory_or_register(first_byte: u8) -> bool {
    let mask_res = first_byte & MASKS::IMMEDIATE_TO_MEMORY_OR_REGISTER.bits();
    return mask_res == IMMEDIATE_TO_MEMORY_OR_REGISTER::MOV.bits()
        || mask_res == IMMEDIATE_TO_MEMORY_OR_REGISTER::SUB_OR_CMP_OR_ADD.bits();
}

fn is_memory_mode_direct(operation: Operation, second_byte: u8) -> bool {
    let rm_res = second_byte & MASKS::RM.bits();
    // When mod == 00 (memory mode, no displacement) and RM == 110 then memory mode is direct with 16-bit displacement.
    if operation == Operation::MEMORY_MODE_NONE && rm_res == 0b_00_000_110 {
        return true;
    }
    return false;
}

// determines the operation and then fetches the mnemonic used (E.g. add, mov etc.)
fn determine_operation(first_byte: u8, second_byte: u8) -> Instruction {
    let mod_res = second_byte & MASKS::MOD.bits();
    let mut operation = MOD_RESULTS::convert_to_operation(mod_res);
    let is_word_size = is_word_size(first_byte, operation);

    // When mod == 00 and RM == 110 then memory mode is direct with 16-bit displacement.
    if is_memory_mode_direct(operation, second_byte) {
        operation = Operation::MEMORY_MODE_DIRECT;
    }

    if is_mov_immediate_to_register(first_byte) {
        if is_word_size {
            return Instruction {
                op: Operation::IMMEDIATE_TO_REGISTER_MOV_16,
                mnemonic: "mov",
            };
        } else {
            return Instruction {
                op: Operation::IMMEDIATE_TO_REGISTER_MOV_8,
                mnemonic: "mov",
            };
        }
    } else if is_register_or_memory_with_register(first_byte) {
        let mnemonic = REG_OR_MEMORY_WITH_REGISTER_MNEMONIC_RESULTS::convert_to_mnemonic(
            first_byte & MASKS::REG_OR_MEMORY_WITH_REGISTER.bits(),
        );
        return Instruction {
            op: operation,
            mnemonic: mnemonic,
        };
    } else if is_immediate_to_memory_or_register(first_byte) {
        let mask_res = first_byte & MASKS::IMMEDIATE_TO_MEMORY_OR_REGISTER.bits();
        let mnemonic = IMMEDIATE_TO_MEMORY_OR_REGISTER::convert_to_mnemonic(mask_res, second_byte);

        if mod_res == MOD_RESULTS::MEMORY_MODE_8_BIT_DISPLACEMENT.bits() {
            operation = Operation::IMMEDIATE_TO_MEMORY_8;
        } else if mod_res == MOD_RESULTS::MEMORY_MODE_16_BIT_DISPLACEMENT.bits() {
            operation = Operation::IMMEDIATE_TO_MEMORY_16;
        } else if mod_res == MOD_RESULTS::REGISTER_MODE.bits() {
            if is_word_size {
                operation = Operation::IMMEDIATE_TO_REGISTER_16;
            } else {
                operation = Operation::IMMEDIATE_TO_REGISTER_8;
            }
        }
        return Instruction {
            op: operation,
            mnemonic: mnemonic,
        };
    } else {
        panic!("unsupported operation {:08b}", first_byte)
    }
}

fn get_register(get_reg: bool, op: Operation, first_byte: u8, second_byte: u8) -> &'static str {
    let rm_res = second_byte & MASKS::RM.bits();
    let reg_res = second_byte & MASKS::REG.bits();
    let is_word_size = is_word_size(first_byte, op);

    if get_reg {
        match (reg_res, is_word_size) {
            (0b_00_000_000, true) => "ax",
            (0b_00_001_000, true) => "cx",
            (0b_00_010_000, true) => "dx",
            (0b_00_011_000, true) => "bx",
            (0b_00_100_000, true) => "sp",
            (0b_00_101_000, true) => "bp",
            (0b_00_110_000, true) => "si",
            (0b_00_111_000, true) => "di",
            //
            (0b_00_000_000, false) => "al",
            (0b_00_001_000, false) => "cl",
            (0b_00_010_000, false) => "dl",
            (0b_00_011_000, false) => "bl",
            (0b_00_100_000, false) => "ah",
            (0b_00_101_000, false) => "ch",
            (0b_00_110_000, false) => "dh",
            (0b_00_111_000, false) => "bh",
            _ => panic!("unknown register - get_register - get_reg branch\nreg was: {:08b}, first_byte was: {:08b}, second_byte was: {:08b}", reg_res, first_byte, second_byte),
        }
    } else {
        if op == Operation::REGISTER_MODE {
            // 11
            match (rm_res, is_word_size) {
                (0b_00_000_000, true) => "ax",
                (0b_00_000_001, true) => "cx",
                (0b_00_000_010, true) => "dx",
                (0b_00_000_011, true) => "bx",
                (0b_00_000_100, true) => "sp",
                (0b_00_000_101, true) => "bp",
                (0b_00_000_110, true) => "si",
                (0b_00_000_111, true) => "di",
                //
                (0b_00_000_000, false) => "al",
                (0b_00_000_001, false) => "cl",
                (0b_00_000_010, false) => "dl",
                (0b_00_000_011, false) => "bl",
                (0b_00_000_100, false) => "ah",
                (0b_00_000_101, false) => "ch",
                (0b_00_000_110, false) => "dh",
                (0b_00_000_111, false) => "bh",
                _ => panic!("unknown register - get_register - Operation::REGISTER_MODE\nreg was: {:08b}, first_byte was: {:08b}, second_byte was: {:08b}", reg_res, first_byte, second_byte),
            }
        } else if op == Operation::MEMORY_MODE_NONE {
            // 10/01/00
            match rm_res {
                0b_00_000_000 => "bx + si",
                0b_00_000_001 => "bx + di",
                0b_00_000_010 => "bp + si",
                0b_00_000_011 => "bp + di",
                0b_00_000_100 => "si",
                0b_00_000_101 => "di",
                0b_00_000_110 => panic!(
                    "This: {:08b} should never be hit because it's handled by the direct memory operation.", rm_res),
                0b_00_000_111 => "bx",
                _ => panic!("unknown register - get_register - Operation::MEMORY_MODE_NONE\n R/M was: {:08b}, first_byte was: {:08b}, second_byte was: {:08b}", rm_res, first_byte, second_byte),
            }
        } else if op == Operation::MEMORY_MODE_8_BIT_DISPLACEMENT {
            match rm_res {
                // D8 stand for 8-bit diacement. We will be search & replacing the D8 string with the 8-bit displacement.
                0b_00_000_000 => "bx + si + ", // d8 gets added on all of these
                0b_00_000_001 => "bx + di + ",
                0b_00_000_010 => "bp + si + ",
                0b_00_000_011 => "bp + di + ",
                0b_00_000_100 => "si + ",
                0b_00_000_101 => "di + ",
                0b_00_000_110 => "bp + ",
                0b_00_000_111 => "bx + ",
                _ => panic!(
                    "unknown register - get_register - Operation::MEMORY_MODE_8_BIT_DISPLACEMENT\n R/M was: {:08b}, first_byte was: {:08b}, second_byte was: {:08b}", rm_res, first_byte, second_byte
                ),
            }
        }
        // else if op == Operation::IMMEDIATE_TO_REGISTER_16 {
        // }
        else if op == Operation::MEMORY_MODE_16_BIT_DISPLACEMENT {
            match rm_res {
                // D8 stand for 8-bit displacement. We will be search & replacing the D8 string with the 8-bit displacement.
                0b_00_000_000 => "bx + si + ", // d16 gets added into all of these instructions.
                0b_00_000_001 => "bx + di + ",
                0b_00_000_010 => "bp + si + ",
                0b_00_000_011 => "bp + di + ",
                0b_00_000_100 => "si + ",
                0b_00_000_101 => "di + ",
                0b_00_000_110 => "bp + ",
                0b_00_000_111 => "bx + ",
                _ => panic!(
                    "unknown register - get_register - Operation::MEMORY_MODE_16_BIT_DISPLACEMENT\n R/M was: {:08b}, first_byte was: {:08b}, second_byte was: {:08b}", rm_res, first_byte, second_byte
                ),
            }
        } else if op == Operation::MEMORY_MODE_DIRECT {
            // 00 + 110 RM
            "" // we return an empty string because MEMORY_MODE_DIRECT does not have a register, instead it's a direct 16-bit address that will be fetched later.
        } else {
            panic!("Unsupported operation - get_register")
        }
    }
}

fn reg_is_dest(byte: u8) -> bool {
    return byte & FIRST_BYTE::D_BIT_MASK.bits() != 0;
}

// sig stand for significant
fn combine_bytes(most_sig_byte: u8, least_sig_byte: u8) -> u16 {
    let combined_bytes: u16 = ((most_sig_byte as u16) << 8) | (least_sig_byte as u16);
    return combined_bytes;
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let binary_path = &args[1];
    let binary_contents = fs::read(binary_path).unwrap();

    let mut i: usize = 0;
    while i < binary_contents.len() {
        let first_byte = binary_contents[i];
        let second_byte = binary_contents[i + 1];

        let instruction = determine_operation(first_byte, second_byte);

        let reg_register = get_register(true, instruction.op, first_byte, second_byte);
        let rm_register = get_register(false, instruction.op, first_byte, second_byte);

        println!(
            "Mnemonic: {}, reg_register: {}, rm_register: {}",
            instruction.mnemonic, reg_register, rm_register
        );

        i += 2
    }
}
//   mov   _DW_MOD_REG_R/M
//0b_100010_10_11_010_010

// MOV 100010
// ADD 000000
// SUB 001010
// CMP 001110
//
// IMMEDIATES MOD (check both bytes each time in case)
// MOV 000
// ADD 000
// SUB 101
// CMP 111

#[cfg(test)]
mod tests {
    use super::*;

    struct determine_operation_params {
        first_byte: u8,
        second_byte: u8,
        expected_op: Operation,
        expected_mnemonic: &'static str,
    }

    #[test]
    fn test_determine_operation() {
        type D = determine_operation_params;
        let params: [D; 6] = [
            D {
                first_byte: 0b_10001000,
                second_byte: 0b_11_000_000,
                expected_op: Operation::REGISTER_MODE,
                expected_mnemonic: "mov",
            },
            D {
                first_byte: 0b_00101000,
                second_byte: 0b_11_111_000,
                expected_op: Operation::REGISTER_MODE,
                expected_mnemonic: "sub",
            },
            D {
                first_byte: 0b_10000000,
                second_byte: 0b_00_111_000,
                expected_op: Operation::MEMORY_MODE_NONE,
                expected_mnemonic: "cmp",
            },
            D {
                first_byte: 0b_10000000,
                second_byte: 0b_00_111_110,
                expected_op: Operation::MEMORY_MODE_DIRECT,
                expected_mnemonic: "cmp",
            },
            D {
                first_byte: 0b_10000000,
                second_byte: 0b_01_101_000,
                expected_op: Operation::IMMEDIATE_TO_MEMORY_8,
                expected_mnemonic: "sub",
            },
            D {
                first_byte: 0b_10000000,
                second_byte: 0b_10_101_000,
                expected_op: Operation::IMMEDIATE_TO_MEMORY_16,
                expected_mnemonic: "sub",
            },
        ];

        let mut i = 0;
        for param in params {
            let op = determine_operation(param.first_byte, param.second_byte);
            assert_eq!(param.expected_op, op.op, "unexpected val at {}", i);
            assert_eq!(
                param.expected_mnemonic, op.mnemonic,
                "unexpected val at {}",
                i
            );
            i += 1
        }
    }

    struct get_register {
        get_reg: bool,
        op: Operation,
        first_byte: u8,
        second_byte: u8,
        expected_register: &'static str,
    }
    #[test]
    fn test_get_register() {
        type D = get_register;
        let params: [D; 6] = [
            D {
                get_reg: false,
                op: Operation::REGISTER_MODE,
                first_byte: 0b_00_000_001,
                second_byte: 0b_00_000_000,
                expected_register: "ax",
            },
            D {
                get_reg: false,
                op: Operation::MEMORY_MODE_NONE,
                first_byte: 0b_00_000_000,
                second_byte: 0b_00_000_001,
                expected_register: "bx + di",
            },
            D {
                get_reg: false,
                op: Operation::MEMORY_MODE_8_BIT_DISPLACEMENT,
                first_byte: 0b_00_000_000,
                second_byte: 0b_00_000_110,
                expected_register: "bp + ",
            },
            D {
                get_reg: false,
                op: Operation::MEMORY_MODE_16_BIT_DISPLACEMENT,
                first_byte: 0b_00_000_000,
                second_byte: 0b_00_000_110,
                expected_register: "bp + ",
            },
            D {
                get_reg: true,
                op: Operation::MEMORY_MODE_16_BIT_DISPLACEMENT,
                first_byte: 0b_00_000_000,
                second_byte: 0b_00_000_000,
                expected_register: "al",
            },
            D {
                get_reg: false,
                op: Operation::MEMORY_MODE_DIRECT,
                first_byte: 0b_00_000_000,
                second_byte: 0b_00_000_000,
                expected_register: "",
            },
        ];
        for param in params {
            assert_eq!(
                get_register(param.get_reg, param.op, param.first_byte, param.second_byte),
                param.expected_register
            )
        }
    }
}
