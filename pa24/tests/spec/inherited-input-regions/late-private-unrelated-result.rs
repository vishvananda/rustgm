fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn second<'a,'b,T>(pair:(&'a T,&'b T))->&'b T{pair.1}fn f<'a,'b,T>(x:&'a T,y:&'b T)->&'a T{let c=|pair:(&T,_)|second(pair);c((x,y))}fn main(){}
