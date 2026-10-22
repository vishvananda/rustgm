fn main(){let mut a=[37u32,41];let p=&raw mut a[0];unsafe{core::ptr::copy(p,p,2);}check(a[0]==37&&a[1]==41);}
