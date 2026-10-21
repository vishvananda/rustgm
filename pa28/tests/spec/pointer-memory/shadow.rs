mod core{pub mod ptr{pub fn read(x:u32)->u32{x}pub fn write(x:u32,y:u32)->u32{x+y}}}const X:u32=7;fn main(){let _=core::ptr::read(X);let _=core::ptr::write(3,4);}
