use core::arch::asm;
pub fn f(p:*mut u8){unsafe{asm!("/* {p} */",p=in(reg)p,options(nomem,nostack,preserves_flags));}}
