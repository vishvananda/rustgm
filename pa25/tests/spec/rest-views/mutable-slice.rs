const fn change(x:&mut[u8]){match x{[_,r @ ..,_]=>{r[0]=37;r[1]=41;},_=>()}}const N:u8={let mut a=[2u8,3,5,7];change(&mut a);a[1]+a[2]};fn main(){let _=N;}
