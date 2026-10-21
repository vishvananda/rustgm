const X:bool={let mut x=false;let p=(&raw mut x) as *mut u8;unsafe{*p=2;}x};fn main(){let _=X;}
