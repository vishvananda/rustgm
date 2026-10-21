mod std{pub mod ptr{pub fn read_volatile(x:u32)->u32{x+1}}}fn f()->u32{std::ptr::read_volatile(7)}fn main(){}
