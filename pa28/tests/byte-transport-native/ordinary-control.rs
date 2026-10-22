const X:u32={let mut source=[1u32,2,3,4];unsafe{core::ptr::copy(&raw const source[0],&raw mut source[1],3);}source[3]};
fn main(){check(X==3);}
