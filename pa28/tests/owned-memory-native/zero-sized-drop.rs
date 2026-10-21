static mut N:u32=0;struct D;impl Drop for D{fn drop(&mut self){unsafe{N+=1;}}}fn main(){{let mut d=D;unsafe{core::ptr::write(&raw mut d,D);}check(unsafe{N==0});}check(unsafe{N==1});}
