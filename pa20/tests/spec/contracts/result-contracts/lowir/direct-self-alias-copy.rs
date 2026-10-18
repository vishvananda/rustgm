fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
type Id<T>=T;trait C where Id<Self>:Copy{}impl C for u8{}fn dup<T:C>(x:T)->(T,T){(x,x)}fn main(){let p=dup(7u8);check(p.0==7&&p.1==7);}