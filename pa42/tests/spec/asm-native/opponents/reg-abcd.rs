use core::arch::asm;
pub fn f(p:*mut u8){unsafe{asm!("/* {p} */",p=in(reg_abcd)p,options(nostack,preserves_flags));}}
