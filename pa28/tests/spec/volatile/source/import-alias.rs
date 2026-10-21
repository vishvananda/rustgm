use core::ptr::read_volatile as read;fn f(p:*const u32)->u32{unsafe{read(p)}}fn main(){}
