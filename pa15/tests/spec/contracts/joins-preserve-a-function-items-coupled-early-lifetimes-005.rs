fn a(x:&u8)->&u8{x}fn b(x:&u8)->&u8{x}fn f(x:&u8,c:bool)->&u8{let p=if c{a}else{b};p(x)}
fn main() {}
