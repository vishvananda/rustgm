fn f(p:*const u32,b:bool)->u32{let r;if b{r=core::ptr::read_volatile::<u32>;}unsafe{r(p)}}fn main(){}
