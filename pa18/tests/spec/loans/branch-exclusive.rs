fn f<T>(x:&mut T,b:bool)->&mut T{if b{&mut*x}else{&mut*x}}
fn main(){}
