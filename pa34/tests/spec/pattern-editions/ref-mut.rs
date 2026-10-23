fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut values=[7u8];{let [ref mut x]=&mut values;*x=9;}check(values[0]==9);}
