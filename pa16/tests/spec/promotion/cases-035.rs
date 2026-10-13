struct S(u8);fn f(c:bool)->&'static S{&S(if c{1}else{2})}
fn main() {}
