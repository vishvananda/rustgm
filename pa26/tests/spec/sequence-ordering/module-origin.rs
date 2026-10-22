fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
mod inner{pub fn compare(a:[u8;2],b:[u8;2])->bool{a<b}}fn main(){check(inner::compare([1,2],[1,3]));}