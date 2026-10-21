struct P{a:u32}fn main(){let mut p:P;unsafe{core::ptr::write(&raw mut p,P{a:37});}}
