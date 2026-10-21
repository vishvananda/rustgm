struct P{a:u32}fn main(){let mut p=P{a:7};let q=P{a:37};unsafe{core::ptr::write(&raw mut p,q);}let _r=q;}
