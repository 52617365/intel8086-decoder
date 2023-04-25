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
    if op == Operation::IMMEDIATE_TO_REGISTER_16 || op == Operation::IMMEDIATE_TO_REGISTER_8 {
        return first_byte & MASKS::MOV_IMMEDIATE_TO_REGISTER_W_BIT.bits() != 0;
    } else {
        return first_byte & MASKS::W_BIT.bits() != 0;
    }
}

fn is_immediate_to_register(first_byte: u8) -> bool {
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

// determines the operation and then fetches the mnemonic used (E.g. add, mov etc.)
fn determine_operation(first_byte: u8, second_byte: u8) -> Instruction {
    let mod_res = second_byte & MASKS::MOD.bits();
    let rm_res = second_byte & MASKS::RM.bits();
    let mut operation = MOD_RESULTS::convert_to_operation(mod_res);

    // When mod == 00 and RM == 110 then memory mode is direct with 16-bit displacement.
    if operation == Operation::MEMORY_MODE_NONE && rm_res == 0b_00_000_110 {
        operation = Operation::MEMORY_MODE_DIRECT;
    }

    if is_immediate_to_register(first_byte) {
        if first_byte & MASKS::MOV_IMMEDIATE_TO_REGISTER_W_BIT.bits() != 0 {
            return Instruction {
                op: Operation::IMMEDIATE_TO_REGISTER_16,
                mnemonic: "mov",
            };
        } else {
            return Instruction {
                op: Operation::IMMEDIATE_TO_REGISTER_8,
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
        return Instruction {
            op: operation,
            mnemonic: mnemonic,
        };
    } else {
        panic!("unsupported operation {:08b}", first_byte)
    }
}

// fn get_register(get_reg: bool, ) -> {

// }
struct get_register_params {
    first_byte: u8,
    second_byte: u8,
    get_reg_register: bool, // true if reg, false if r/m
    op: MOD_RESULTS,
    rm: u8,
}
// fn get_register(params: get_register_params) -> &'static str {

//     if params.get_reg_register {
//     } else {
//         match params.op {
//             MOD_RESULTS::MEMORY_MODE => {
//                 // 11
//                 match params.rm {
// 0b_00_000_000 =>

//                 }
//             }
//         }
//     }
// }
// fn get_register(
//     first_byte: u8,
//     second_byte: u8,
//     get_reg: bool,
//     instruction: Instruction,
// ) -> &'static str {
//     match (get_reg, instruction.operation) {
//         (true, Operation::IMMEDIATE_TO_REGISTER_16)
//         | (true, Operation::IMMEDIATE_TO_REGISTER_8)
//         | (false, Operation::REGISTER_MODE) => {
//             let mask_result = first_byte & FIRST_BYTE::IMMEDIATE_OR_REGISTER_MODE_REG_MASK.bits();
//             let mask_cast = IMMEDIATE_TO_REGISTER_MODE_REG_MASK_RESULTS::from_bits(mask_result)
//                 .expect("expected bitflag to contain value but it didn't");

//             return match (instruction.is_word_size, mask_cast) {
//                 (true, IMMEDIATE_TO_REGISTER_MODE_REG_MASK_RESULTS::AX_OR_AL) => "ax",
//                 (true, IMMEDIATE_TO_REGISTER_MODE_REG_MASK_RESULTS::CX_OR_CL) => "cx",
//                 (true, IMMEDIATE_TO_REGISTER_MODE_REG_MASK_RESULTS::DX_OR_DL) => "dx",
//                 (true, IMMEDIATE_TO_REGISTER_MODE_REG_MASK_RESULTS::BX_OR_BL) => "bx",
//                 (true, IMMEDIATE_TO_REGISTER_MODE_REG_MASK_RESULTS::SP_OR_AH) => "sp",
//                 (true, IMMEDIATE_TO_REGISTER_MODE_REG_MASK_RESULTS::BP_OR_CH) => "bp",
//                 (true, IMMEDIATE_TO_REGISTER_MODE_REG_MASK_RESULTS::SI_OR_DH) => "si",
//                 (true, IMMEDIATE_TO_REGISTER_MODE_REG_MASK_RESULTS::DI_OR_BH) => "di",
//                 //
//                 (false, IMMEDIATE_TO_REGISTER_MODE_REG_MASK_RESULTS::AX_OR_AL) => "al",
//                 (false, IMMEDIATE_TO_REGISTER_MODE_REG_MASK_RESULTS::CX_OR_CL) => "cl",
//                 (false, IMMEDIATE_TO_REGISTER_MODE_REG_MASK_RESULTS::DX_OR_DL) => "dl",
//                 (false, IMMEDIATE_TO_REGISTER_MODE_REG_MASK_RESULTS::BX_OR_BL) => "bl",
//                 (false, IMMEDIATE_TO_REGISTER_MODE_REG_MASK_RESULTS::SP_OR_AH) => "ah",
//                 (false, IMMEDIATE_TO_REGISTER_MODE_REG_MASK_RESULTS::BP_OR_CH) => "ch",
//                 (false, IMMEDIATE_TO_REGISTER_MODE_REG_MASK_RESULTS::SI_OR_DH) => "dh",
//                 (false, IMMEDIATE_TO_REGISTER_MODE_REG_MASK_RESULTS::DI_OR_BH) => "bh",
//                 _ => panic!("Unknown register"),
//             };
//         }
//         (true, _) => {
//             // REG REGISTERS
//             let mask_result = REGISTER_TO_OR_MEMORY_REG_MASK_RESULTS::from_bits(
//                 second_byte & SECOND_BYTE::REGISTER_TO_OR_MEMORY_REG_MASK.bits(),
//             )
//             .expect("expected bits but it contained none.");
//             return match (instruction.is_word_size, mask_result) {
//                 (true, REGISTER_TO_OR_MEMORY_REG_MASK_RESULTS::AX_OR_AL) => "ax",
//                 (true, REGISTER_TO_OR_MEMORY_REG_MASK_RESULTS::CX_OR_CL) => "cx",
//                 (true, REGISTER_TO_OR_MEMORY_REG_MASK_RESULTS::DX_OR_DL) => "dx",
//                 (true, REGISTER_TO_OR_MEMORY_REG_MASK_RESULTS::BX_OR_BL) => "bx",
//                 (true, REGISTER_TO_OR_MEMORY_REG_MASK_RESULTS::SP_OR_AH) => "sp",
//                 (true, REGISTER_TO_OR_MEMORY_REG_MASK_RESULTS::BP_OR_CH) => "bp",
//                 (true, REGISTER_TO_OR_MEMORY_REG_MASK_RESULTS::SI_OR_DH) => "si",
//                 (true, REGISTER_TO_OR_MEMORY_REG_MASK_RESULTS::DI_OR_BH) => "di",
//                 //
//                 (false, REGISTER_TO_OR_MEMORY_REG_MASK_RESULTS::AX_OR_AL) => "al",
//                 (false, REGISTER_TO_OR_MEMORY_REG_MASK_RESULTS::CX_OR_CL) => "cl",
//                 (false, REGISTER_TO_OR_MEMORY_REG_MASK_RESULTS::DX_OR_DL) => "dl",
//                 (false, REGISTER_TO_OR_MEMORY_REG_MASK_RESULTS::BX_OR_BL) => "bl",
//                 (false, REGISTER_TO_OR_MEMORY_REG_MASK_RESULTS::SP_OR_AH) => "ah",
//                 (false, REGISTER_TO_OR_MEMORY_REG_MASK_RESULTS::BP_OR_CH) => "ch",
//                 (false, REGISTER_TO_OR_MEMORY_REG_MASK_RESULTS::SI_OR_DH) => "dh",
//                 (false, REGISTER_TO_OR_MEMORY_REG_MASK_RESULTS::DI_OR_BH) => "bh",
//                 _ => panic!("Unknown register"),
//             };
//         }

//         (false, _) => {
//             // This uses the reg field from mov immediate to register, the reg field in this instruction
//             // is in the first byte when normally its in the second byte.
//             let res = IMMEDIATE_TO_REGISTER_REG_FIELD_MASK_RESULTS::from_bits(
//                 first_byte & FIRST_BYTE::IMMEDIATE_OR_REGISTER_MODE_REG_MASK.bits(),
//             )
//             .expect("expected bits but there were none.");
//             return match res {
//                 IMMEDIATE_TO_REGISTER_REG_FIELD_MASK_RESULTS::BX_PLUS_SI => "bx + si",
//                 IMMEDIATE_TO_REGISTER_REG_FIELD_MASK_RESULTS::BX_PLUS_DI => "bx + di",
//                 IMMEDIATE_TO_REGISTER_REG_FIELD_MASK_RESULTS::BP_PLUS_SI => "bp + si",
//                 IMMEDIATE_TO_REGISTER_REG_FIELD_MASK_RESULTS::BP_PLUS_DI => "bp + di",
//                 IMMEDIATE_TO_REGISTER_REG_FIELD_MASK_RESULTS::SI => "si",
//                 IMMEDIATE_TO_REGISTER_REG_FIELD_MASK_RESULTS::DI => "di",
//                 IMMEDIATE_TO_REGISTER_REG_FIELD_MASK_RESULTS::BP_OR_NONE => {
//                     if instruction.operation == Operation::MEMORY_MODE_DIRECT {
//                         return "bp";
//                     } else {
//                         return "";
//                     }
//                 }
//                 IMMEDIATE_TO_REGISTER_REG_FIELD_MASK_RESULTS::BX => "bx",
//                 _ => panic!("unknown instruction detected"),
//             };
//         }
//         _ => panic!("Unknown instruction"),
//     }
// }

// #[derive(Clone, Copy)]
// struct Instruction {
//     mnemonic: &'static str,
//     operation: Operation,
//     is_word_size: bool,
// }
// In this function we have to check both the first byte and second byte because the first byte determines the contents of the second byte.
// TODO: we actually don't even need to get the word size because it's already handled but because some code relies on it we still keep it.

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

        let op = determine_operation(first_byte, second_byte);

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
        let params: [D; 4] = [
            D {
                first_byte: 0b_10000000,
                second_byte: 0b_11_000_000,
                expected_op: Operation::REGISTER_MODE,
                expected_mnemonic: "add",
            },
            D {
                first_byte: 0b_10000000,
                second_byte: 0b_11_111_000,
                expected_op: Operation::REGISTER_MODE,
                expected_mnemonic: "cmp",
            },
            D {
                first_byte: 0b_10000000,
                second_byte: 0b_00_111_000,
                expected_op: Operation::MEMORY_MODE_NONE,
                expected_mnemonic: "cmp",
            },
            D {
                // TODO: add memory mode direct.
                first_byte: 0b_10000000,
                second_byte: 0b_00_111_110,
                expected_op: Operation::MEMORY_MODE_DIRECT,
                expected_mnemonic: "cmp",
            },
        ];

        for param in params {
            let op = determine_operation(param.first_byte, param.second_byte);
            assert_eq!(param.expected_op, op.op);
            assert_eq!(param.expected_mnemonic, op.mnemonic);
        }
    }
}
