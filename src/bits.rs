use bitflags::bitflags;
bitflags! {
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
        const MEMORY_MODE = 0b_00_000_000; // (Only if r/m is not 110, then its a 16 bit placement).
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
    }

    pub struct FIRST_BYTE: u8 {
        const IMMEDIATE_OR_REGISTER_MODE_REG_MASK = 0b_00_000_111;
        const D_BIT_MASK = 0b000000_10; // This is the D bit specified after the instruction operand. It's responsible for specifying the destination and source register.
        const IMMEDIATE_TO_REGISTER_W_MASK = 0b_0000_1000; // This is the W bit of a memory to register, register to memory and register to register move and it's responsible for determining the size of the registers (8 or 16 bit).
        const MEMORY_TO_REGISTER_VICA_VERCA_W_MASK = 0b000000_01; // This is the W bit of a memory to register, register to memory and register to register move and it's responsible for determining the size of the registers (8 or 16 bit).
        const IMMEDIATE_TO_REGISTER_REG_FIELD_MASK = 0b_00_000_111; // this is used to get the contents of the REG field if it's present in the first byte.
    }

    pub struct SECOND_BYTE: u8 {
        const REGISTER_TO_OR_MEMORY_REG_MASK = 0b_00_111_000; // this is only used for register to register / memory to register and vica verca operations.
        const MOD_MASK = 0b_11_000_000;
        const RM_MASK = 0b_00_000_111; // this is used to get the contents of the R/M field
    }

}
