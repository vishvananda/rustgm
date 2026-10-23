const N:u8={let b=[65u8];let s:&[u8]=&b;let p=s as *const [u8] as *mut str;let m=unsafe{&mut *p};unsafe{*(m as *mut str as *mut u8)=66;}b[0]};fn main(){let _=N;}
