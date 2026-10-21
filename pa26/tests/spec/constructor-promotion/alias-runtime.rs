use core::mem::ManuallyDrop as M;fn f()->&'static M<u32>{&M::new(7)}fn main(){}
