#[repr(C)]struct P{a:u32,b:u32}fn main(){let mut a=[3u32,7,11,13];unsafe{core::ptr::copy((&raw const a[0]) as *const P,(&raw mut a[1]) as *mut P,1);}check(a[0]==3&&a[1]==3&&a[2]==7&&a[3]==13);}
