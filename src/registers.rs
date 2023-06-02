// TODO: Next we need to emulate the instructions, this means that we need to keep state of the values passed into the registers/memory locations.
// it could possibly be wise to create a vector of structs that contain the register name, and the value associated with it.
// we can then do a linear loop through the vector to see if the register we care about matches and get the value out that way.
// it could also have a field that is like og_value that gets updated each time the value gets changed.

// we could construct this struct for each register at the start and then just iterate over the collection again and again.
struct Register {
    register: &'static str,
    updated_value: u16,
    original_value: u16,
}

// TODO: add all the possible registers in some constant array and then iterate over it in this function and construct a Register struct from each one, appending it to the vector.
// We then linearly search through the vector to find the register we care about and get the value out of it.
// This is done so that we can keep a state of the value of the register.
// we want to emulate the registers and their values during moves etc.
pub fn construct_registers() -> Vec<Register>{
    // give me all possible 16 and 8 bit registers.
    let mut registers: Vec<Register> = Vec::with_capacity();
    todo!()
}