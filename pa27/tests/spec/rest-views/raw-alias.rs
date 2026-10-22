const N:u8={let mut a=[2u8,3,5,7];{let [_,ref mut r @ ..,_]=a;let p=&raw mut r[0];unsafe{*p=37;}r[1]=41;}a[1]+a[2]};fn main(){let _=N;}
