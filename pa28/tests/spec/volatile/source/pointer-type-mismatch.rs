fn f(p:*const i32)->u32{unsafe{core::ptr::read_volatile::<u32>(p)}}fn main(){}
