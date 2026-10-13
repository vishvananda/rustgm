fn change(p:&mut u8)->bool{*p=1;true}fn f(p:&mut (u8,u8)){match *p{(0,_) if change(&mut p.0)=>(),_=>()}}
fn main() {}
