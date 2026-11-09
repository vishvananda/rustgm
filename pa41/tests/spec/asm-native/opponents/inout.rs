use core::arch::asm;
pub fn f(mut x:u64)->u64{unsafe{asm!("add {x},1",x=inout(reg)x,options(nostack));}x}
