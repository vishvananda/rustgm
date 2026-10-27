fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let n=match '\u{10ffff}'{char::MAX=>1,_=>2};let z=match '\0'{char::MIN=>3,_=>4};check(n==1);check(z==3);let q=match 'x'{char::MIN..=char::MAX=>5,_=>6};check(q==5);}
