use core::arch::asm;
pub fn f(p:*mut u8){unsafe{asm!("jmp 2f",".ascii \"a//b/*c\"","2:","# {p}",p=in(reg)p,options(nostack,preserves_flags));}}
