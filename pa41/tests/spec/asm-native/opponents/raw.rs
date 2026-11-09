use core::arch::asm;
pub fn f(){unsafe{asm!("nop",options(raw,nostack,preserves_flags));}}
