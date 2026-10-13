fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<'a>{x:&'a [u8],y:u64}fn f(x:S<'_>)->S<'_>{x}fn main(){let a=[7,9];let s=f(S{x:&a,y:42});check(s.x[1]==9 && s.y==42);}
