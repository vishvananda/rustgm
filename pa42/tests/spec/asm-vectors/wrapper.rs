use core::arch::asm;
#[derive(Clone,Copy)]struct V([f32;4]);fn f(x:V){unsafe{asm!("/* {0} */",in(xmm_reg)x);}}
