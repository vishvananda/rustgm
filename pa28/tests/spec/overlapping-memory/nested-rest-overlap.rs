const N:[u8;5]={let mut a=[2u8,3,5,7,11];{let[_,ref mut r @ ..]=a;let[ref mut s @ ..,_]=*r;unsafe{core::ptr::copy(&raw const s[0],&raw mut s[1],2);}}a};fn main(){let _=N;}
