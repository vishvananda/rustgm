#[repr(C)]struct P{a:u32,b:u32}fn main(){let mut a=[3u32,7,11,13];unsafe{core::ptr::copy((&raw const a[1]) as *const P,(&raw mut a[0]) as *mut P,1);}check(a[0]==7&&a[1]==11&&a[2]==11&&a[3]==13);}
