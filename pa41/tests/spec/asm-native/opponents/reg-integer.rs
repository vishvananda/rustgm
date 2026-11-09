use core::arch::asm;
pub fn f(x:u64){unsafe{asm!("/* {x} */",x=in(reg)x,options(nostack,preserves_flags));}}
