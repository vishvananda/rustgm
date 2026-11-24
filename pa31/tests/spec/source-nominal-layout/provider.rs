#[repr(C, align(32))]
pub struct Aligned(pub u8);

#[repr(C, packed(2))]
pub struct Packed {
    pub first: u8,
    pub rest: u32,
}

#[repr(C)]
#[rustc_pass_indirectly_in_non_rustic_abis]
pub struct Indirect(pub u64);

pub fn aligned(value: u8) -> Aligned {
    Aligned(value)
}

pub fn packed() -> Packed {
    Packed { first: 1, rest: 34 }
}

