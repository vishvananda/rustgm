use core::arch::asm;
pub fn f(p:*mut u8){unsafe{asm!("/* rdi */",in("rdi")p,options(nostack,preserves_flags));}}
