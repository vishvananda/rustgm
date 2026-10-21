extern crate core as c;fn f(p:*const u32)->u32{unsafe{c::ptr::read_volatile(p)}}fn main(){}
