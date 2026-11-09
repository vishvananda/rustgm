use core::arch::asm;
pub fn f()->!{unsafe{asm!("ud2",options(noreturn));}}
