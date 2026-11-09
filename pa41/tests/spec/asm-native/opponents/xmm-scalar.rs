use core::arch::asm;
pub fn f(x:f32){unsafe{asm!("/* {x} */",x=in(xmm_reg)x,options(nostack,preserves_flags));}}
