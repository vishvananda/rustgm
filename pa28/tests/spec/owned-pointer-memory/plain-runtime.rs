struct P{a:u32}fn main(){let mut p=P{a:7};unsafe{let q=core::ptr::read(&raw const p);core::ptr::write(&raw mut p,P{a:q.a+30});}}
