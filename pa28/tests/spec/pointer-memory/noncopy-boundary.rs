struct P{a:u32}fn main(){let x=P{a:37};unsafe{let y=core::ptr::read(&raw const x);let _=y.a;}}
