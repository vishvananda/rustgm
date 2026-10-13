fn f<'a>(p:&'a u8)->&'static u8{if false{&*p}else{loop{}}}
fn main() {}
