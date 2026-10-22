fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f<'a>(a:[&'a u8;1],b:[&'static u8;1])->&'static u8{let _=a<b;a[0]}fn main(){}