const A:[u8;2]={let [_,r @ ..,_]=*b"abcd";r};const N:u8={let mut a=A;a[0]=37;a[0]+A[0]};fn main(){let _=N;}
