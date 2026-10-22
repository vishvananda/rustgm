fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}const N:u32={let x=[1u32,7];let p=(&x as *const [u32;2])as *mut u32;unsafe{*p.add(1)=11;}x[1]};fn main(){let _=N;}
