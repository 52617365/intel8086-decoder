use std::num::Wrapping;
use std::ops::Add;

// Flags registers that will be used to determine the state of the program.
const FLAGS_REGISTERS: [&str; 2] = [
    // "cf", // carry flag
    // "reserved1",
    // "pf", // parity flag
    // "reserved2",
    // "af", // auxiliary carry flag
    // "reserved3",
    "zf", // zero flag
    "sf", // sign flag
    // "tf", // trap flag
    // "if", // interrupt enable flag
    // "df", // direction flag
    // "of", // overflow flag
    // "iopl", // i/o privilege level
    // "nt", // nested task flag
    // "reserved4",
    // "rf", // resume flag
    // "vm", // virtual 8086 mode flag
    // "ac", // alignment check
    // "vif", // virtual interrupt flag
    // "vip", // virtual interrupt pending
    // "id", // able to use cpuid instruction
    // "reserved5",
    // "reserved6",
];

pub struct FlagRegister {
    pub register: &'static str,
    pub is_set: bool,
    pub mask: usize,
}

pub fn construct_flag_registers() -> [FlagRegister; 2] {
    let flags = [
        // FlagRegister { register: "CF", is_set: false, mask: 0x0001 },
        // FlagRegister { register: "PF", is_set: false, mask: 0x0004 },
        // FlagRegister { register: "AF", is_set: false, mask: 0x0010 },
        FlagRegister { register: "ZF", is_set: false, mask: 0x0040 },
        FlagRegister { register: "SF", is_set: false, mask: 0x0080 },
        // FlagRegister { register: "TF", is_set: false, mask: 0x0100 },
        // FlagRegister { register: "IF", is_set: false, mask: 0x0200 },
        // FlagRegister { register: "DF", is_set: false, mask: 0x0400 },
        // FlagRegister { register: "OF", is_set: false, mask: 0x0800 },
        // FlagRegister { register: "IOPL", is_set: false, mask: 0x3000 },
        // FlagRegister { register: "NT", is_set: false, mask: 0x4000 },
        // FlagRegister { register: "MD", is_set: false, mask: 0x8000 },
        // FlagRegister { register: "RF", is_set: false, mask: 0x0001_0000 },
        // FlagRegister { register: "VM", is_set: false, mask: 0x0002_0000 },
        // FlagRegister { register: "AC", is_set: false, mask: 0x0004_0000 },
        // FlagRegister { register: "VIF", is_set: false, mask: 0x0008_0000 },
        // FlagRegister { register: "VIP", is_set: false, mask: 0x0010_0000 },
        // FlagRegister { register: "ID", is_set: false, mask: 0x0020_0000 },
        // FlagRegister { register: "AI", is_set: false, mask: 0x8000_0000 },
    ];
    return flags;
}

pub fn set_is_set_for_flag_register(flag: &'static str, flag_registers: &mut [FlagRegister], value: bool) -> () {
    for flag_register in flag_registers.iter_mut() {
        if flag_register.register == flag {
                flag_register.is_set = value;
                return
            }
        }
    panic!("Flag {} not found", flag);
}

pub fn set_flags(destination_value: usize, flag_registers: &mut [FlagRegister], is_word_size: bool) -> () {
    if destination_value == 0 {
        set_is_set_for_flag_register("ZF", flag_registers, true);
        return
    } else {
        set_is_set_for_flag_register("ZF", flag_registers, false);
    }
    if number_is_signed(destination_value, is_word_size) {
        set_is_set_for_flag_register("SF", flag_registers, true);
        return
    } else {
        set_is_set_for_flag_register("SF", flag_registers, false);
        return
    }
}

pub fn get_all_currently_set_flags(flag_registers: &[FlagRegister]) -> Vec<&str> {
    let mut flags: Vec<&str> = Vec::with_capacity(flag_registers.len());
    for flag_register in flag_registers.iter() {
        if flag_register.is_set {
            flags.push(flag_register.register);
        }
    }
    return flags;
}

fn number_is_signed(value: usize, is_word_size: bool) -> bool {
    let highest_bit = get_highest_bit(value, is_word_size);
    return highest_bit == 1
}

fn get_highest_bit(value: usize, is_word_size: bool) -> usize {
    if is_word_size {
        // println!("value: {:16b}", value);
        return value << 15;
    } else {
        // println!("value: {:08b}", value);
        return value << 7;
    }
}
