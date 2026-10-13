fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
macro_rules! choose{($x:expr)=>{if let n@0..=9=$x&&n>0{n}else{99}}}fn main(){check(choose!(7u8)==7&&choose!(0u8)==99);}