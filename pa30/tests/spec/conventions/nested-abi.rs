extern "C" fn f(){}fn main(){let p:extern "C" fn()=f;let _: &fn()=&p;}
