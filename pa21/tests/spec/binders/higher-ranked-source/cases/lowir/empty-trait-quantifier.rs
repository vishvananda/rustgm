fn f<T:for<> Copy>(x:T)->(T,T){(x,x)}fn main(){let p=f(7u8);if p.0!=p.1{loop{}}}
