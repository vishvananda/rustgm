use core::arch::asm;
pub fn f(p:*mut u8){unsafe{asm!("# {{ {p} }}","nop",p=in(reg)p,options(nostack,preserves_flags));}}
