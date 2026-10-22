fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn identity<T>(x:T)->T{x}fn id<'a,'b>(_:&'a u8,b:&'b u8)->&'b u8{b}fn main(){let a=3;let r;{let b=4;r=identity(id)(&a,&b);}let _=*r;}
