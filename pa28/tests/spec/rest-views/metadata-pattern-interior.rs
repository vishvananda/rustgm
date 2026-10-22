use core::sync::atomic::AtomicU32;const fn count(a:&[AtomicU32])->u8{match a{[_,..]=>1,_=>0}}static A:[AtomicU32;1]=[AtomicU32::new(0)];const N:u8=count(&A);fn main(){let _=N;}
