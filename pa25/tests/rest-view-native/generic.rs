const fn mid<T:Copy>(a:[T;4])->[T;2]{let[_,r @ ..,_]=a;r}const A:[u32;2]=mid([2,3,5,7]);fn main(){check(A[0]==3);check(A[1]==5);let r=mid([11u64,13,17,19]);check(r[0]==13);check(r[1]==17);}
