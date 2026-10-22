const N:u8={let a=[2u8,3,5];let [_,ref r @ ..]=a;let p=&raw const r[0] as *mut u8;unsafe{*p=37;}r[0]};fn main(){let _=N;}
