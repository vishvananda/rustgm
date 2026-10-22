fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn eq(a:&[u8],b:&[u8])->bool{a==b}fn main(){let a=[9u8,1,2,8];let a:&[u8]=&a;match a{[_,rest @ ..,_]=>check(eq(rest,&[1,2])),_=>check(false)}}
