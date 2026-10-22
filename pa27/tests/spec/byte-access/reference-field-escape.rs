#[repr(C)]struct P{a:u16,b:u16}const R:&P={let x=0x00070001u32;unsafe{&*(&raw const x as *const P)}};const N:u16=R.b;fn main(){let _=N;}
