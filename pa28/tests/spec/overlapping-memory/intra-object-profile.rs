#[repr(C)]struct P{a:u32,b:u32}const N:u32={let mut a=[2u32,3,5,7];unsafe{core::ptr::copy((&raw const a[0]) as *const P,(&raw mut a[1]) as *mut P,1);}a[2]};fn main(){let _=N;}
