fn f<'a>(x:&'a u8)->impl Copy+'a{x}fn take<'a,T:'a>(_:T,_:&'a u8){}fn main(){let a=17;take(f(&a),&a);}
