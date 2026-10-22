#[repr(align(16))]struct Z;const N:()=unsafe{core::ptr::copy_nonoverlapping(1usize as *const Z,16usize as *mut Z,0)};fn main(){let _=N;}
