fn f<'x,T>(x:(&'x u8,T))->((&'x u8,T),(&'x u8,T))where for<'a>(&'a u8,T):Copy{(x,x)}fn main(){let v=7;let p=f((&v,3u8));if p.0.1!=p.1.1{loop{}}}
