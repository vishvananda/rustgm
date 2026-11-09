use core::arch::asm;
pub fn f(p:*mut u8)->u64{let y:u64;unsafe{asm!("mov {y},{p}",p=in(reg)p,y=out(reg)y,options(pure,nomem,nostack));}y}
