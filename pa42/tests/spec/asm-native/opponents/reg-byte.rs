use core::arch::asm;
pub fn f(b:u8){unsafe{asm!("/* {b} */",b=in(reg_byte)b,options(nostack,preserves_flags));}}
