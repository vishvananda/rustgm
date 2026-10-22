fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f<'a,'b>(left:[&mut &'a u8;1],right:[&mut &'b u8;1])->bool{left<right}fn main(){}