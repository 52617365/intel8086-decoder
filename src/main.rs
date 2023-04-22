mod bits;
use bits::*;

use core::panic;
use std::{env, fs};

// Ideas:
// Should for example the registers be coupled with the Operation enum? E.g. the reg and r/m registers would be wrapped into the enum itself, this would
// allow stuff to not be so independent from eachother.

// All the different instruction operations we're looking to handle at the moment.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Operation {
    REGISTER_MODE,            // no displacement
    IMMEDIATE_TO_REGISTER_8, // The first byte is set to 10110... and the instruction is 2 bytes wide. (last byte is the immediate)
    IMMEDIATE_TO_REGISTER_16, // The first byte is set to 10111... and the instruction is 3 bytes wide. (last 2 bytes is the immediate)
    //
    MEMORY_MODE_8,      // 8 bit displacement
    MEMORY_MODE_16,     // 16 bit displacement
    MEMORY_MODE_NONE,   // No displacement expect if R/M is 110, then it's 16 bit direct address.
    MEMORY_MODE_DIRECT, // This is mod 00 with r/m 110 16 bit displacement into a direct memory address
}

fn get_register(
    first_byte: u8,
    second_byte: u8,
    get_reg: bool,
    instruction: Instruction,
) -> &'static str {
    match (get_reg, instruction.operation) {
        (true, Operation::IMMEDIATE_TO_REGISTER_16)
        | (true, Operation::IMMEDIATE_TO_REGISTER_8)
        | (false, Operation::REGISTER_MODE) => {
            let mask_result = first_byte & FIRST_BYTE::IMMEDIATE_OR_REGISTER_MODE_REG_MASK.bits();
            let mask_cast = IMMEDIATE_TO_REGISTER_MODE_REG_MASK_RESULTS::from_bits(mask_result)
                .expect("expected bitflag to contain value but it didn't");

            return match (instruction.is_word_size, mask_cast) {
                (true, IMMEDIATE_TO_REGISTER_MODE_REG_MASK_RESULTS::AX_OR_AL) => "ax",
                (true, IMMEDIATE_TO_REGISTER_MODE_REG_MASK_RESULTS::CX_OR_CL) => "cx",
                (true, IMMEDIATE_TO_REGISTER_MODE_REG_MASK_RESULTS::DX_OR_DL) => "dx",
                (true, IMMEDIATE_TO_REGISTER_MODE_REG_MASK_RESULTS::BX_OR_BL) => "bx",
                (true, IMMEDIATE_TO_REGISTER_MODE_REG_MASK_RESULTS::SP_OR_AH) => "sp",
                (true, IMMEDIATE_TO_REGISTER_MODE_REG_MASK_RESULTS::BP_OR_CH) => "bp",
                (true, IMMEDIATE_TO_REGISTER_MODE_REG_MASK_RESULTS::SI_OR_DH) => "si",
                (true, IMMEDIATE_TO_REGISTER_MODE_REG_MASK_RESULTS::DI_OR_BH) => "di",
                //
                (false, IMMEDIATE_TO_REGISTER_MODE_REG_MASK_RESULTS::AX_OR_AL) => "al",
                (false, IMMEDIATE_TO_REGISTER_MODE_REG_MASK_RESULTS::CX_OR_CL) => "cl",
                (false, IMMEDIATE_TO_REGISTER_MODE_REG_MASK_RESULTS::DX_OR_DL) => "dl",
                (false, IMMEDIATE_TO_REGISTER_MODE_REG_MASK_RESULTS::BX_OR_BL) => "bl",
                (false, IMMEDIATE_TO_REGISTER_MODE_REG_MASK_RESULTS::SP_OR_AH) => "ah",
                (false, IMMEDIATE_TO_REGISTER_MODE_REG_MASK_RESULTS::BP_OR_CH) => "ch",
                (false, IMMEDIATE_TO_REGISTER_MODE_REG_MASK_RESULTS::SI_OR_DH) => "dh",
                (false, IMMEDIATE_TO_REGISTER_MODE_REG_MASK_RESULTS::DI_OR_BH) => "bh",
                _ => panic!("Unknown register"),
            };
        }
        (true, _) => {
            // REG REGISTERS
            let mask_result = REGISTER_TO_OR_MEMORY_REG_MASK_RESULTS::from_bits(
                second_byte & SECOND_BYTE::REGISTER_TO_OR_MEMORY_REG_MASK.bits(),
            )
            .expect("expected bits but it contained none.");
            return match (instruction.is_word_size, mask_result) {
                (true, REGISTER_TO_OR_MEMORY_REG_MASK_RESULTS::AX_OR_AL) => "ax",
                (true, REGISTER_TO_OR_MEMORY_REG_MASK_RESULTS::CX_OR_CL) => "cx",
                (true, REGISTER_TO_OR_MEMORY_REG_MASK_RESULTS::DX_OR_DL) => "dx",
                (true, REGISTER_TO_OR_MEMORY_REG_MASK_RESULTS::BX_OR_BL) => "bx",
                (true, REGISTER_TO_OR_MEMORY_REG_MASK_RESULTS::SP_OR_AH) => "sp",
                (true, REGISTER_TO_OR_MEMORY_REG_MASK_RESULTS::BP_OR_CH) => "bp",
                (true, REGISTER_TO_OR_MEMORY_REG_MASK_RESULTS::SI_OR_DH) => "si",
                (true, REGISTER_TO_OR_MEMORY_REG_MASK_RESULTS::DI_OR_BH) => "di",
                //
                (false, REGISTER_TO_OR_MEMORY_REG_MASK_RESULTS::AX_OR_AL) => "al",
                (false, REGISTER_TO_OR_MEMORY_REG_MASK_RESULTS::CX_OR_CL) => "cl",
                (false, REGISTER_TO_OR_MEMORY_REG_MASK_RESULTS::DX_OR_DL) => "dl",
                (false, REGISTER_TO_OR_MEMORY_REG_MASK_RESULTS::BX_OR_BL) => "bl",
                (false, REGISTER_TO_OR_MEMORY_REG_MASK_RESULTS::SP_OR_AH) => "ah",
                (false, REGISTER_TO_OR_MEMORY_REG_MASK_RESULTS::BP_OR_CH) => "ch",
                (false, REGISTER_TO_OR_MEMORY_REG_MASK_RESULTS::SI_OR_DH) => "dh",
                (false, REGISTER_TO_OR_MEMORY_REG_MASK_RESULTS::DI_OR_BH) => "bh",
                _ => panic!("Unknown register"),
            };
        }

        (false, _) => {
            // This uses the reg field from mov immediate to register, the reg field in this instruction
            // is in the first byte when normally its in the second byte.
            let res = IMMEDIATE_TO_REGISTER_REG_FIELD_MASK_RESULTS::from_bits(
                first_byte & FIRST_BYTE::IMMEDIATE_OR_REGISTER_MODE_REG_MASK.bits(),
            )
            .expect("expected bits but there were none.");
            return match res {
                IMMEDIATE_TO_REGISTER_REG_FIELD_MASK_RESULTS::BX_PLUS_SI => "bx + si",
                IMMEDIATE_TO_REGISTER_REG_FIELD_MASK_RESULTS::BX_PLUS_DI => "bx + di",
                IMMEDIATE_TO_REGISTER_REG_FIELD_MASK_RESULTS::BP_PLUS_SI => "bp + si",
                IMMEDIATE_TO_REGISTER_REG_FIELD_MASK_RESULTS::BP_PLUS_DI => "bp + di",
                IMMEDIATE_TO_REGISTER_REG_FIELD_MASK_RESULTS::SI => "si",
                IMMEDIATE_TO_REGISTER_REG_FIELD_MASK_RESULTS::DI => "di",
                IMMEDIATE_TO_REGISTER_REG_FIELD_MASK_RESULTS::BP_OR_NONE => {
                    if instruction.operation == Operation::MEMORY_MODE_DIRECT {
                        return "bp";
                    } else {
                        return "";
                    }
                }
                IMMEDIATE_TO_REGISTER_REG_FIELD_MASK_RESULTS::BX => "bx",
                _ => panic!("unknown instruction detected"),
            };
        }
        _ => panic!("Unknown instruction"),
    }
}

#[derive(Clone, Copy)]
struct Instruction {
    mnemonic: &'static str,
    operation: Operation,
    is_word_size: bool,
}
// In this function we have to check both the first byte and second byte because the first byte determines the contents of the second byte.
fn get_instruction(first_byte: u8, second_byte: u8) -> Instruction {
    let immediate_to_reg: u8 = first_byte & OPERATIONS::IMMEDIATE_TO_REGISTER_MASK.bits();
    let immediate_to_reg_casted = IMMEDIATE_TO_REGISTER_MASK_RESULTS::from_bits(immediate_to_reg);

    if let Some(casted_val) = immediate_to_reg_casted {
        match casted_val {
            IMMEDIATE_TO_REGISTER_MASK_RESULTS::IMMEDIATE_TO_REGISTER_16 => {
                return Instruction {
                    // 16 bit immediate to register because first byte is different from others and w bit is set to 1.
                    operation: Operation::IMMEDIATE_TO_REGISTER_16,
                    mnemonic: "mov",
                    is_word_size: first_byte & FIRST_BYTE::IMMEDIATE_TO_REGISTER_W_MASK.bits() != 0,
                };
            }
            IMMEDIATE_TO_REGISTER_MASK_RESULTS::IMMEDIATE_TO_REGISTER_8 => {
                return Instruction {
                    // 8 bit immediate to register because first byte is different from others and w bit is set to 0.
                    operation: Operation::IMMEDIATE_TO_REGISTER_8,
                    mnemonic: "mov",
                    is_word_size: first_byte & FIRST_BYTE::IMMEDIATE_TO_REGISTER_W_MASK.bits() != 0,
                };
            }
            _ => (), // we want to continue to the next branch.
        }
    }

    // "expected cast to contain correct bits but it didnt. First byte was: {:08b} and second byte was: {:08b}, it contained: {:08b} after bit manipulations.",
    // first_byte, second_byte, immediate_to_reg
    let mod_results = MOD_MODE_RESULTS::from_bits(second_byte & SECOND_BYTE::MOD_MASK.bits())
        .expect("expected bitflag to contain value but it didn't");

    return match mod_results {
        MOD_MODE_RESULTS::REGISTER_MODE => {
            let mask_res = MEMORY_TO_REGISTER_VICA_VERCA_MNEMONIC_MASK_RESULTS::from_bits(
                first_byte & FIRST_BYTE::MEMORY_TO_REGISTER_VICA_VERCA_MNEMONIC_MASK.bits(),
            )
            .expect("expected bits but got none");

            let mnemonic = match mask_res {
                MEMORY_TO_REGISTER_VICA_VERCA_MNEMONIC_MASK_RESULTS::MOV => "mov",
                MEMORY_TO_REGISTER_VICA_VERCA_MNEMONIC_MASK_RESULTS::ADD => "add",
                MEMORY_TO_REGISTER_VICA_VERCA_MNEMONIC_MASK_RESULTS::SUB => "sub",
                MEMORY_TO_REGISTER_VICA_VERCA_MNEMONIC_MASK_RESULTS::CMP => "cmp",
                _ => panic!("unsupported mnemonic: {:08b}", mask_res),
            };
            Instruction {
                operation: Operation::REGISTER_MODE,
                mnemonic: mnemonic,
                is_word_size: first_byte & FIRST_BYTE::MEMORY_TO_REGISTER_VICA_VERCA_W_MASK.bits()
                    != 0,
            }
        }
        MOD_MODE_RESULTS::MEMORY_MODE_8 => Instruction {
            operation: Operation::MEMORY_MODE_8,
            mnemonic: "mov",
            is_word_size: first_byte & FIRST_BYTE::MEMORY_TO_REGISTER_VICA_VERCA_W_MASK.bits() != 0,
        },
        MOD_MODE_RESULTS::MEMORY_MODE_16 => Instruction {
            operation: Operation::MEMORY_MODE_16,
            mnemonic: "mov",
            is_word_size: first_byte & FIRST_BYTE::MEMORY_TO_REGISTER_VICA_VERCA_W_MASK.bits() != 0,
        },
        MOD_MODE_RESULTS::MEMORY_MODE => {
            // we are masking the R/M bits here because (MOD = 00 + R/M 110) = 16 bit displacement.
            let res = second_byte & SECOND_BYTE::RM_MASK.bits();
            if res == 0b_00_000_110 {
                Instruction {
                    operation: Operation::MEMORY_MODE_DIRECT,
                    mnemonic: "mov",
                    is_word_size: first_byte
                        & FIRST_BYTE::MEMORY_TO_REGISTER_VICA_VERCA_W_MASK.bits()
                        != 0,
                }
            } else {
                Instruction {
                    operation: Operation::MEMORY_MODE_NONE,
                    mnemonic: "mov",
                    is_word_size: first_byte
                        & FIRST_BYTE::MEMORY_TO_REGISTER_VICA_VERCA_W_MASK.bits()
                        != 0,
                }
            }
        }
        _ => panic!("Unknown operation - get_operation line: {}", line!()),
    };
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

        let instruction = get_instruction(first_byte, second_byte);

        let reg_register = get_register(first_byte, second_byte, true, instruction);
        let rm_register = get_register(first_byte, second_byte, false, instruction);

        let mut disp: Option<usize> = match instruction.operation {
            Operation::MEMORY_MODE_8 => {
                let displacement = binary_contents[i + 2];
                i += 1; // adding one to not go off course in the loop.
                Some(displacement as usize)
            }
            Operation::MEMORY_MODE_16 | Operation::MEMORY_MODE_DIRECT => {
                let third_byte = binary_contents[i + 2];
                let fourth_byte = binary_contents[i + 3];
                let combined_bytes: u16 = combine_bytes(fourth_byte, third_byte);
                i += 2; // adding two to not go off course in the loop. Because we went forward 2x with the third and fourth_byte index.

                Some(combined_bytes as usize)
            }
            Operation::IMMEDIATE_TO_REGISTER_16 => {
                let third_byte = binary_contents[i + 2];
                let combined_bytes: u16 = combine_bytes(third_byte, second_byte);

                i += 1; // adding one to not go off course in the loop. Because we went forward with the third_byte index.

                Some(combined_bytes as usize)
            }
            Operation::IMMEDIATE_TO_REGISTER_8 => Some(second_byte as usize),
            Operation::REGISTER_MODE | Operation::MEMORY_MODE_NONE => None,
        };

        // Handling the case where for example there is a displacement like mov [bp + 0], ch which is an useless displacement.
        if disp == Some(0) {
            disp = None;
        }

        let reg_is_dest = reg_is_dest(first_byte);
        // When dealing immediate to register instructions, reg is always on the lefthand side so we don't have to check for it.
        // We are also unwrapping disp because we have covered the cases on the previous branch and are sure that it contains a value.
        format_results(instruction, reg_register, disp, reg_is_dest, rm_register);
        i += 2; // each iteration is 1 byte, a instruction is minimum 2 bytes.
    }
}

fn format_results(
    instruction: Instruction,
    reg_register: &str,
    disp: Option<usize>,
    reg_is_dest: bool,
    rm_register: &str,
) {
    if instruction.operation == Operation::IMMEDIATE_TO_REGISTER_8
        || instruction.operation == Operation::IMMEDIATE_TO_REGISTER_16
    {
        println!(
            "{} {}, {}",
            instruction.mnemonic,
            reg_register,
            disp.expect("unwrapped disp because we thought we were sure it had a value inside.")
        );
    } else {
        match (reg_is_dest, disp) {
            (true, Some(disp)) => {
                if instruction.operation == Operation::MEMORY_MODE_DIRECT {
                    println!("{} {}, [{}]", instruction.mnemonic, reg_register, disp);
                } else {
                    println!(
                        "{} {}, [{} + {}]",
                        instruction.mnemonic, reg_register, rm_register, disp
                    );
                }
            }
            (false, Some(disp)) => {
                println!(
                    "{} [{} + {}], {}",
                    instruction.mnemonic, rm_register, disp, reg_register
                );
            }
            (true, None) => {
                if instruction.operation == Operation::REGISTER_MODE {
                    println!("{} {}, {}", instruction.mnemonic, reg_register, rm_register);
                } else {
                    println!(
                        "{} {}, [{}]",
                        instruction.mnemonic, reg_register, rm_register
                    );
                }
            }
            (false, None) => {
                if instruction.operation == Operation::REGISTER_MODE {
                    println!("{} {}, {}", instruction.mnemonic, rm_register, reg_register);
                } else {
                    println!(
                        "{} [{}], {}",
                        instruction.mnemonic, rm_register, reg_register
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reg_is_dest() {
        let true_byte: u8 = 0b000000_10;
        let false_byte: u8 = 0b000000_00;
        assert_eq!(reg_is_dest(true_byte), true);
        assert_eq!(reg_is_dest(false_byte), false);
    }

    struct get_register_params {
        first_byte: u8,
        second_byte: u8,
        instruction: Instruction,
        expected_result: &'static str,
        get_reg: bool,
    }

    #[test]
    fn test_get_register() {
        let params: [get_register_params; 8] = [
            get_register_params {
                first_byte: 0b_11_111_000,
                second_byte: 0b_00_000_000,
                instruction: Instruction {
                    mnemonic: "",
                    operation: Operation::IMMEDIATE_TO_REGISTER_16,
                    is_word_size: true,
                },
                expected_result: "ax",
                get_reg: true,
            },
            get_register_params {
                first_byte: 0b_11_111_001,
                second_byte: 0b_00_000_000,
                instruction: Instruction {
                    mnemonic: "",
                    operation: Operation::IMMEDIATE_TO_REGISTER_16,
                    is_word_size: false,
                },
                expected_result: "cl",
                get_reg: true,
            },
            get_register_params {
                first_byte: 0b_11_111_001,
                second_byte: 0b_00_000_000,
                instruction: Instruction {
                    mnemonic: "",
                    operation: Operation::REGISTER_MODE,
                    is_word_size: false,
                },
                expected_result: "cl",
                get_reg: false,
            },
            get_register_params {
                first_byte: 0b_11_111_000,
                second_byte: 0b_00_000_000,
                instruction: Instruction {
                    mnemonic: "",
                    operation: Operation::MEMORY_MODE_16,
                    is_word_size: false,
                },
                expected_result: "bx + si",
                get_reg: false,
            },
            get_register_params {
                first_byte: 0b_11_111_110,
                second_byte: 0b_00_000_000,
                instruction: Instruction {
                    mnemonic: "",
                    operation: Operation::MEMORY_MODE_DIRECT,
                    is_word_size: false,
                },
                expected_result: "bp",
                get_reg: false,
            },
            get_register_params {
                first_byte: 0b_11_111_110,
                second_byte: 0b_00_000_000,
                instruction: Instruction {
                    mnemonic: "",
                    operation: Operation::MEMORY_MODE_16,
                    is_word_size: false,
                },
                expected_result: "",
                get_reg: false,
            },
            get_register_params {
                first_byte: 0b_00_000_000,
                second_byte: 0b_00_111_000,
                instruction: Instruction {
                    mnemonic: "",
                    operation: Operation::MEMORY_MODE_16,
                    is_word_size: false,
                },
                expected_result: "bh",
                get_reg: true,
            },
            get_register_params {
                first_byte: 0b_00_000_000,
                second_byte: 0b_00_111_000,
                instruction: Instruction {
                    mnemonic: "",
                    operation: Operation::MEMORY_MODE_16,
                    is_word_size: true,
                },
                expected_result: "di",
                get_reg: true,
            },
        ];

        for param in params {
            assert_eq!(
                get_register(
                    param.first_byte,
                    param.second_byte,
                    param.get_reg,
                    param.instruction
                ),
                param.expected_result
            );
        }
    }

    struct get_instruction_params {
        first_byte: u8,
        second_byte: u8,
        expected_op: Operation,
        expected_mnemonic: &'static str,
    }

    #[test]
    fn test_get_instruction() {
        let params: [get_instruction_params; 10] = [
            get_instruction_params {
                first_byte: 0b_1011_1000,
                second_byte: 0b_0000_0000,
                expected_op: Operation::IMMEDIATE_TO_REGISTER_16,
                expected_mnemonic: "mov",
            },
            get_instruction_params {
                first_byte: 0b_1011_0000,
                second_byte: 0b_0000_0000,
                expected_op: Operation::IMMEDIATE_TO_REGISTER_8,
                expected_mnemonic: "mov",
            },
            get_instruction_params {
                first_byte: 0b_0000_00_00,
                second_byte: 0b_11_001_010,
                expected_op: Operation::REGISTER_MODE,
                expected_mnemonic: "add",
            },
            get_instruction_params {
                first_byte: 0b_00_10_1000,
                second_byte: 0b_11_001_010,
                expected_op: Operation::REGISTER_MODE,
                expected_mnemonic: "sub",
            },
            get_instruction_params {
                first_byte: 0b_00_11_1000,
                second_byte: 0b_11_001_010,
                expected_op: Operation::REGISTER_MODE,
                expected_mnemonic: "cmp",
            },
            get_instruction_params {
                first_byte: 0b_0000_00_00,
                second_byte: 0b_01_000_000,
                expected_op: Operation::MEMORY_MODE_8,
                expected_mnemonic: "mov",
            },
            get_instruction_params {
                first_byte: 0b_0000_00_00,
                second_byte: 0b_10_000_000,
                expected_op: Operation::MEMORY_MODE_16,
                expected_mnemonic: "mov",
            },
            get_instruction_params {
                first_byte: 0b_0000_00_00,
                second_byte: 0b_00_000_000,
                expected_op: Operation::MEMORY_MODE_NONE,
                expected_mnemonic: "mov",
            },
            get_instruction_params {
                first_byte: 0b_0000_00_00,
                second_byte: 0b_00_000_110,
                expected_op: Operation::MEMORY_MODE_DIRECT,
                expected_mnemonic: "mov",
            },
            get_instruction_params {
                // TODO
                first_byte: 0b_00_000_000,
                second_byte: 0b_11_111_111,
                expected_op: Operation::REGISTER_MODE,
                expected_mnemonic: "add",
            },
        ];
        let mut i = 0;
        for param in params {
            let instruction = get_instruction(param.first_byte, param.second_byte);

            assert_eq!(instruction.operation, param.expected_op);
            assert_eq!(
                instruction.mnemonic, param.expected_mnemonic,
                "Expected the mnemonic to be {} but it was {}, the first byte was: {:08b} and the second byte was {:08b}, happened at {}", param.expected_mnemonic, instruction.mnemonic, param.first_byte,param.second_byte, i);
            i += 1
        }
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
