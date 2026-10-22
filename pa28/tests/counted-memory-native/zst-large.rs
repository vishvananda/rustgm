#[repr(align(16))]struct Z;fn main(){let a=Z;let mut b=Z;unsafe{core::ptr::copy_nonoverlapping(&raw const a,&raw mut b,18446744073709551615usize);}check(true);}
