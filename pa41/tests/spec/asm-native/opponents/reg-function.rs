use core::arch::asm;
fn g(){}pub fn f(){let h:fn()=g;unsafe{asm!("/* {h} */",h=in(reg)h,options(nostack,preserves_flags));}}
