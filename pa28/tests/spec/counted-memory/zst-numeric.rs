#[repr(align(16))]struct Z;const N:()=unsafe{core::ptr::copy_nonoverlapping(16usize as *const Z,16usize as *mut Z,18446744073709551615usize)};fn main(){let _=N;}
