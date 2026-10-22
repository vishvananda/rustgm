const fn change(x:&mut[u8]){match x{[_,r @ ..]=>match r{[_,s @ ..]=>s[0]=37,_=>()},_=>()}}const N:u8={let mut a=[2u8,3,5,7];change(&mut a);a[2]};fn main(){let _=N;}
