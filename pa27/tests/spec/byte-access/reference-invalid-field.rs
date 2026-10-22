#[repr(C)]struct P{a:bool,b:u8}static X:u16=2;const R:&P=unsafe{&*(&raw const X as *const P)};const N:u8=R.b;fn main(){let _=N;}
