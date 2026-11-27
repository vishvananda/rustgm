use core::arch::asm;
pub fn f(){unsafe{asm!("xor eax,eax",out("eax")_,options(nostack));}}
