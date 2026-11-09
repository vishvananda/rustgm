use core::arch::asm;
#[inline]pub unsafe fn f(p:*mut f32){unsafe{asm!("prefetchnta [{p}]",p=in(reg)p,options(nostack,preserves_flags));}}
