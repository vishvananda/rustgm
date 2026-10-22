fn check(b:bool){if !b{let z=0;let _=1/z;}}fn f<'a,T>(s:&'a[T])->Option<&'a T>{match s{[x,..]=>Some(x),_=>None}}fn main(){let a=[1u8,2];match f(&a){Some(x)=>check(*x==1),None=>check(false)}}
