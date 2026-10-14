struct S(u8);fn twice<T:Copy>(x:T)->(T,T){(x,x)}fn main(){let _=twice(S(7));}
