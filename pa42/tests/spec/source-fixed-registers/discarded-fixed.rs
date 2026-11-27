pub fn discarded_fixed<T>(value:u32)->u32 {
    unsafe { asm!("xor eax, eax", inout("eax")value=>_, options(nostack)); }
    value
}
