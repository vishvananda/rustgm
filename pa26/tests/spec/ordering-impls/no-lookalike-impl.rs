fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
enum Ordering{Less,Equal}fn main(){let _=Ordering::Less==Ordering::Equal;}