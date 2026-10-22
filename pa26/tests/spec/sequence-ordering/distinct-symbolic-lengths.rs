fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f<const N:usize,const M:usize>(a:[u8;N],b:[u8;M])->bool{a<b}fn main(){}