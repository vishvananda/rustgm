struct S{x:u32}fn main(){let mut s=S{x:7};unsafe{core::ptr::write_volatile(&raw mut s,S{x:9});}}
