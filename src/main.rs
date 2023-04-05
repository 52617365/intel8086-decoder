use std::{env, fs};
const MOV_BYTES: usize = 6;
const FULL_INSTRUCTION: usize = 16;

fn get_bits(contents: &[u8]) -> Vec<u8> {
    let mut bits: Vec<u8> = Vec::with_capacity(contents.len() * 8);

    for byte in contents {
        for i in (0..8).rev() {
            let bit = (byte >> i) & 1;
            bits.push(bit);
        }
    }
    return bits;
}

const MOV: usize = 0;
const D: usize = 6;
const W: usize = 7;
const REG_1: usize = 10;
const REG_2: usize = 11;
const REG_3: usize = 12;
const RM_1: usize = 13;
const RM_2: usize = 14;
const RM_3: usize = 15;

fn main() {
    let args: Vec<String> = env::args().collect();
    let binary_path = &args[1];
    let binary_contents = fs::read(binary_path).unwrap();
    let instructions = binary_contents.chunks(2);

    let mut asm: String = String::new();

    let mut REG_IS_SRC = false;
    let mut IS_BYTE_DATA = false;

    let mut SRC_REG: &str;
    let mut DST_REG: &str;

    for inst in instructions {
        let bits = get_bits(&inst);

        if bits[D] == 0 {
            REG_IS_SRC = true;
        } else {
            REG_IS_SRC = false;
        }

        if bits[W] == 0 {
            IS_BYTE_DATA = true;
        } else {
            IS_BYTE_DATA = false;
        }

        if bits[REG_1] == 0 && bits[REG_2] == 0 && bits[REG_3] == 0 {
            if REG_IS_SRC {
                if IS_BYTE_DATA {
                    SRC_REG = "AL";
                } else {
                    SRC_REG = "AX";
                }
            } else {
                if IS_BYTE_DATA {
                    DST_REG = "AL";
                } else {
                    DST_REG = "AX";
                }
            }
        }
        if bits[REG_1] == 0 && bits[REG_2] == 0 && bits[REG_3] == 1 {
            if REG_IS_SRC {
                if IS_BYTE_DATA {
                    SRC_REG = "CL";
                } else {
                    SRC_REG = "CX";
                }
            } else {
                if IS_BYTE_DATA {
                    DST_REG = "CL";
                } else {
                    DST_REG = "CX";
                }
            }
        }
        if bits[REG_1] == 0 && bits[REG_2] == 1 && bits[REG_3] == 0 {
            if REG_IS_SRC {
                if IS_BYTE_DATA {
                    SRC_REG = "DL";
                } else {
                    SRC_REG = "DX";
                }
            } else {
                if IS_BYTE_DATA {
                    DST_REG = "DL";
                } else {
                    DST_REG = "DX";
                }
            }
        }
        if bits[REG_1] == 0 && bits[REG_2] == 1 && bits[REG_3] == 0 {
            if REG_IS_SRC {
                if IS_BYTE_DATA {
                    SRC_REG = "DL";
                } else {
                    SRC_REG = "DX";
                }
            } else {
                if IS_BYTE_DATA {
                    DST_REG = "DL";
                } else {
                    DST_REG = "DX";
                }
            }
        }
        if bits[REG_1] == 0 && bits[REG_2] == 1 && bits[REG_3] == 1 {
            if REG_IS_SRC {
                if IS_BYTE_DATA {
                    SRC_REG = "BL";
                } else {
                    SRC_REG = "BX";
                }
            } else {
                if IS_BYTE_DATA {
                    DST_REG = "BL";
                } else {
                    DST_REG = "BX";
                }
            }
        }
        if bits[REG_1] == 1 && bits[REG_2] == 0 && bits[REG_3] == 0 {
            if REG_IS_SRC {
                if IS_BYTE_DATA {
                    SRC_REG = "AH";
                } else {
                    SRC_REG = "SP";
                }
            } else {
                if IS_BYTE_DATA {
                    DST_REG = "CH";
                } else {
                    DST_REG = "BP";
                }
            }
        }
        if bits[REG_1] == 1 && bits[REG_2] == 0 && bits[REG_3] == 1 {
            if REG_IS_SRC {
                if IS_BYTE_DATA {
                    SRC_REG = "CH";
                } else {
                    SRC_REG = "BP";
                }
            } else {
                if IS_BYTE_DATA {
                    DST_REG = "CH";
                } else {
                    DST_REG = "BP";
                }
            }
        }
        if bits[REG_1] == 1 && bits[REG_2] == 1 && bits[REG_3] == 0 {
            if REG_IS_SRC {
                if IS_BYTE_DATA {
                    SRC_REG = "DH";
                } else {
                    SRC_REG = "SI";
                }
            } else {
                if IS_BYTE_DATA {
                    DST_REG = "DH";
                } else {
                    DST_REG = "SI";
                }
            }
        }
        if bits[REG_1] == 1 && bits[REG_2] == 1 && bits[REG_3] == 1 {
            if REG_IS_SRC {
                if IS_BYTE_DATA {
                    SRC_REG = "BH";
                } else {
                    SRC_REG = "DI";
                }
            } else {
                if IS_BYTE_DATA {
                    DST_REG = "DH";
                } else {
                    DST_REG = "SI";
                }
            }
        }
    }
}
