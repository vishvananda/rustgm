fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
mod local{pub fn same()->bool{[1u8,2]==[1u8,2]}}fn main(){check(local::same())}
