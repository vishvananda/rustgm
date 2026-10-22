const fn middle<T:Copy>(a:[T;4])->[T;2]{let [_,r @ ..,_]=a;r}const A:[u32;2]=middle([2,3,5,7]);fn main(){let _=A;}
