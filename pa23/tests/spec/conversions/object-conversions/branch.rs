trait T{}impl T for u8{}impl T for u16{}fn f<'a>(x:&'a u8,y:&'a u16,b:bool)->&'a dyn T{if b{x}else{y}}
fn main(){}
