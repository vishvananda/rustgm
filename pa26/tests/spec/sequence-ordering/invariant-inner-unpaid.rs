fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f<'a,'b>(x:&'a u8,y:&'b u8){let mut a=x;let mut b=y;let left=[&mut a];let right=[&mut b];let _=left<right;}fn main(){}