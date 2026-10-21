use core::sync::atomic::{AtomicU32 as A,Ordering as O};const fn f<const N:usize>()->u32{let a=A::new(1);if false{a.fetch_xor(1,O::Relaxed)}else{7}}const X:u32=f::<1>();fn main(){let _=X;}
