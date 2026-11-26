pub fn discarded_pointer<T>(value:*mut u8)->*mut u8 {
    unsafe { asm!("mov {p}, 0", p=inout(reg)value=>_, options(nostack,preserves_flags)); }
    value
}
