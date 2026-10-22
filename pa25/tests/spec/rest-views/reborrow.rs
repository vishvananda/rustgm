const fn get(x:&[u8])->u8{x[0]}const fn put(x:&mut[u8]){x[0]=37;}const N:u8={let mut a=[2u8,3,5,7];{let [_,ref mut r @ ..,_]=a;let n=get(r);put(r);r[1]=n;}a[1]+a[2]};fn main(){let _=N;}
