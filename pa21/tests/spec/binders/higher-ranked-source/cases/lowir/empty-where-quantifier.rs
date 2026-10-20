fn f<T>(x:T)->(T,T)where for<> T:Copy{(x,x)}fn main(){let p=f(7u8);if p.0!=p.1{loop{}}}
