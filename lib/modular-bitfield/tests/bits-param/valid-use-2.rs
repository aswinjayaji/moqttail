use modular_bitfield::prelude::*;

#[bitfield(bits = 32)]
#[repr(u32)]
pub struct SignInteger {
    sign: bool,
    value: B31,
}

fn main() {}
