use core::arch::asm;
pub fn f(p:*mut u8){unsafe{asm!("prefetchnta [{p}]",p=in(reg)p,options(readonly,nostack,preserves_flags));}}
