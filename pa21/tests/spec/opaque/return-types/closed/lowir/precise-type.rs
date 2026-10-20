fn make<'a,T:Copy>(x:T,y:&'a u8)->impl Copy+use<T>{x}fn take(x:impl Copy){}fn main(){let x;{let v=7;x=make(11u8,&v);}take(x);}
